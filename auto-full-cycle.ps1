chcp 65001 > $null
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

# ========== 定位项目目录 ==========
$projDir = $PSScriptRoot
if (-not $projDir) { $projDir = Split-Path -Parent $MyInvocation.MyCommand.Path }
if (-not $projDir) { $projDir = Get-Location }
Set-Location $projDir

# ========== 前置检查 ==========
$hasClaude = [bool](Get-Command claude -EA SilentlyContinue)
$hasGit = [bool](Get-Command git -EA SilentlyContinue)
$promptPath = Join-Path $projDir "prompt-full.txt"

if (-not $hasClaude) { Write-Host "错误: 未找到 claude CLI" -ForegroundColor Red; exit 1 }
if (-not (Test-Path $promptPath)) { Write-Host "错误: 未找到 prompt-full.txt" -ForegroundColor Red; exit 1 }

# ========== 加载配置 ==========
$base = Get-Content -Raw -Encoding UTF8 $promptPath
$phaseTimeout = 600
$idleTimeout = 600
$totalStart = Get-Date
$script:trackedFiles = @("RESEARCH.md","PRD.md","IMPLEMENTATION_PLAN.md","TEST_REPORT.md","REVIEW.md","TODO.md")

# ========== 工具函数 ==========
function Log($m) {
    $ts = Get-Date -Format "HH:mm:ss"
    "[$ts] $m" | Tee-Object -FilePath (Join-Path $projDir "dev-agent.log") -Append
}

function Cleanup {
    if ($script:curJob -and $script:curJob.Id) {
        Stop-Job $script:curJob -EA SilentlyContinue
        Remove-Job $script:curJob -Force -EA SilentlyContinue
    }
}

function Run-WithTimeout($cmd, $timeoutSec=120) {
    # 带超时执行命令，避免测试卡死
    $j = Start-Job { param($c) Invoke-Expression $c 2>&1 } -ArgumentList $cmd
    $done = Wait-Job $j -Timeout $timeoutSec
    if ($done) { $out = Receive-Job $j } else { Stop-Job $j; $out = @("超时(${timeoutSec}s)，已跳过") }
    Remove-Job $j -Force -EA SilentlyContinue
    return ($out | Select-Object -Last 30)
}

# ========== 项目检测（启动时一次） ==========
function Detect-Project {
    $info = @{}
    $markers = @{
        "package.json"="Node.js"; "Cargo.toml"="Rust"; "go.mod"="Go"
        "pyproject.toml"="Python"; "requirements.txt"="Python"
        "pom.xml"="Java/Maven"; "build.gradle"="Java/Gradle"; "Gemfile"="Ruby"
    }
    $stacks = @()
    foreach ($m in $markers.Keys) { if (Test-Path $m) { $stacks += $markers[$m] } }
    if (Get-Item *.csproj -EA SilentlyContinue) { $stacks += "C#/.NET" }
    if (Test-Path "src-tauri\Cargo.toml") { $stacks += "Tauri" }
    if ($stacks.Count -gt 0) { $info["stack"] = $stacks -join "+" } else { $info["stack"] = "未知" }

    if (Test-Path "package.json") {
        try { $info["name"] = (Get-Content package.json -Raw -Encoding UTF8 | ConvertFrom-Json).name } catch {}
    }
    if (-not $info["name"]) { $info["name"] = Split-Path $projDir -Leaf }

    foreach ($r in @("README.md","README.rst","README.txt","README")) {
        if (Test-Path $r) { $info["readme"] = (Get-Content $r -Encoding UTF8 | Select-Object -First 10) -join "`n"; break }
    }
    if (Test-Path "CLAUDE.md") { $info["claude"] = (Get-Content CLAUDE.md -Encoding UTF8 | Select-Object -First 15) -join "`n" }
    if (Get-Command gh -EA SilentlyContinue) {
        $gh = gh issue list --limit 5 --state open 2>$null
        if ($LASTEXITCODE -eq 0 -and $gh) { $info["issues"] = $gh }
    }
    return $info
}

# ========== 需求解析 ==========
function Resolve-Requirement {
    param($cliArg, $projInfo)
    if ($cliArg) { $cliArg | Out-File "REQUIREMENT.txt" -Encoding UTF8; return $cliArg }
    if (Test-Path "REQUIREMENT.txt") { $r = Get-Content -Raw -Encoding UTF8 REQUIREMENT.txt; if ($r.Trim()) { return $r.Trim() } }
    if (Test-Path "TODO.md") {
        $todos = Get-Content TODO.md -Encoding UTF8 | Where-Object { $_ -match '^\s*[-*]\s*\[[ x]\]' -and $_ -notmatch '\[x\]' -and $_ -notmatch '\[X\]' }
        if ($todos.Count -gt 0) { Log "从TODO.md提取"; return "继续未完成任务:`n$(($todos | Select-Object -First 5) -join "`n")" }
    }
    Write-Host "`n=== 检测到项目 ===" -ForegroundColor Cyan
    Write-Host "名称: $($projInfo.name)" -ForegroundColor Green
    Write-Host "技术栈: $($projInfo.stack)" -ForegroundColor Green
    Write-Host ""
    $req = Read-Host "输入需求(回车=自动推断)"
    if ($req) { $req | Out-File "REQUIREMENT.txt" -Encoding UTF8; return $req }
    $ctx = "项目: $($projInfo.name) ($($projInfo.stack))"
    if ($projInfo.readme) { $ctx += "`nREADME:`n$($projInfo.readme)" }
    if ($projInfo.claude) { $ctx += "`n规范:`n$($projInfo.claude)" }
    if ($projInfo.issues) { $ctx += "`nIssues:`n$($projInfo.issues)" }
    return "分析项目现状，执行最优先的任务：`n$ctx"
}

# ========== 阶段管理 ==========
function Test-FileValid($path, $minLines=3) {
    # 检查文件存在且有实质内容（不是空文件或只有标题）
    if (-not (Test-Path $path)) { return $false }
    $lines = (Get-Content $path -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 })
    return $lines.Count -ge $minLines
}

function Get-Phase {
    if (Test-Path "PROGRESS.md") {
        $m = (Get-Content PROGRESS.md -Encoding UTF8 | Select-String "^当前阶段:\s*(\d+)").Matches
        if ($m.Groups[1].Value) { return [int]$m.Groups[1].Value }
    }
    # 无PROGRESS.md时，按产出文件推断（必须有实质内容）
    if (Test-FileValid "REVIEW.md" 5)    { return 6 }
    if (Test-FileValid "TEST_REPORT.md" 3) { return 6 }
    if (Test-FileValid "IMPLEMENTATION_PLAN.md" 5) { return 4 }
    if (Test-FileValid "PRD.md" 5)       { return 3 }
    if (Test-FileValid "RESEARCH.md" 3)  { return 2 }
    return 1
}

function Get-FileSnapshot {
    $snap = @{}
    foreach ($f in $script:trackedFiles) { if (Test-Path $f) { $snap[$f] = (Get-FileHash $f -Algorithm MD5).Hash } }
    if ($hasGit) { $head = git rev-parse HEAD 2>$null; if ($head) { $snap["__git__"] = $head } }
    return $snap
}

function Check-Changed($before, $after) {
    foreach ($key in $after.Keys) {
        if ($key -eq "__git__") { if ($before["__git__"] -ne $after["__git__"]) { return $true } }
        else { if (-not $before.ContainsKey($key)) { return $true }; if ($before[$key] -ne $after[$key]) { return $true } }
    }
    foreach ($key in $before.Keys) { if ($key -eq "__git__") { continue }; if (-not $after.ContainsKey($key)) { return $true } }
    return $false
}

function Update-Progress($phase) {
    $nextPhase = $phase + 1
    $files = $script:trackedFiles | Where-Object { Test-Path $_ }
    $diff = ""
    if ($hasGit) {
        $parent = git rev-parse HEAD~1 2>$null
        if ($parent) { $diff = git diff --stat HEAD~1 HEAD 2>$null }
        elseif (git rev-parse HEAD 2>$null) { $diff = git log --oneline -1 2>$null }
    }
    @"
当前阶段: $nextPhase
已完成: 1-$phase
产出: $($files -join ', ')
变更:
$diff
"@ | Out-File "PROGRESS.md" -Encoding UTF8
    Log "Progress: $phase -> $nextPhase"
}

# ========== 本地预处理 ==========

function Get-TestResult {
    $cmd = $null
    if (Test-Path "package.json") {
        try { $pkg = Get-Content package.json -Raw -Encoding UTF8 | ConvertFrom-Json; if ($pkg.scripts.test) { $cmd = "npm test" } } catch {}
    }
    if (-not $cmd -and (Test-Path "Cargo.toml")) { $cmd = "cargo test" }
    if (-not $cmd -and (Test-Path "go.mod")) { $cmd = "go test ./..." }
    if (-not $cmd -and (Test-Path "Makefile")) {
        $mk = Get-Content Makefile -Encoding UTF8 -EA SilentlyContinue
        if ($mk -match '^test:') { $cmd = "make test" }
    }
    if (-not $cmd -and ((Test-Path "pyproject.toml") -or (Test-Path "requirements.txt"))) { $cmd = "pytest --tb=short" }
    if (-not $cmd -and (Get-Item *.sln -EA SilentlyContinue)) { $cmd = "dotnet test" }
    if (-not $cmd -and (Test-Path "pom.xml")) { $cmd = "mvn test -q" }
    if (-not $cmd -and (Test-Path "build.gradle")) { $cmd = "gradle test --quiet" }
    if (-not $cmd) { return "未检测到测试命令，请先编写测试再运行" }
    $result = Run-WithTimeout $cmd 120
    return "$cmd 结果:`n$($result -join "`n")"
}

function Get-BuildResult {
    # 快速编译检查，不跑测试
    $cmd = $null
    if (Test-Path "Cargo.toml") { $cmd = "cargo check" }
    elseif (Test-Path "go.mod") { $cmd = "go build ./..." }
    elseif (Test-Path "package.json") {
        try { $pkg = Get-Content package.json -Raw -Encoding UTF8 | ConvertFrom-Json; if ($pkg.scripts.build) { $cmd = "npm run build" } } catch {}
    }
    elseif (Test-Path "pom.xml") { $cmd = "mvn compile -q" }
    elseif (Test-Path "build.gradle") { $cmd = "gradle compileJava --quiet" }
    elseif (Get-Item *.sln -EA SilentlyContinue) { $cmd = "dotnet build --no-restore -q" }
    elseif ((Test-Path "pyproject.toml") -or (Test-Path "requirements.txt")) { $cmd = "python -m py_compile *.py 2>&1" }
    if (-not $cmd) { return "" }
    $result = Run-WithTimeout $cmd 60
    $ok = $LASTEXITCODE -eq 0
    if ($ok) { return "编译检查通过 ($cmd)" }
    return "编译检查失败 ($cmd):`n$($result -join "`n")"
}

function Get-GitStats {
    if (-not $hasGit) { return "" }
    $log = git log --oneline -10 2>$null
    $stat = git diff --stat HEAD~1 HEAD 2>$null
    if (-not $stat) { $stat = git log --oneline -1 2>$null }
    $branch = git branch --show-current 2>$null
    $out = ""
    if ($branch) { $out += "分支: $branch`n" }
    if ($log) { $out += "最近提交:`n$($log -join "`n")`n" }
    if ($stat) { $out += "最近变更:`n$stat`n" }
    return $out
}

function Get-NextTask {
    if (-not (Test-Path "IMPLEMENTATION_PLAN.md")) { return "" }
    $lines = Get-Content IMPLEMENTATION_PLAN.md -Encoding UTF8
    $pending = $lines | Where-Object { $_ -match '^\s*[-*]\s*\[\s*\]' }
    if ($pending.Count -gt 0) {
        return "下一个待完成任务:`n$($pending[0])`n剩余任务数: $($pending.Count)"
    }
    return "所有任务已完成"
}

function Get-TodoItems {
    if (-not (Test-Path "TODO.md")) { return "" }
    $items = Get-Content TODO.md -Encoding UTF8 | Where-Object { $_ -match '^\s*[-*]\s*\[[ x]\]' -and $_ -notmatch '\[x\]' -and $_ -notmatch '\[X\]' }
    if ($items.Count -gt 0) { return "待办($($items.Count)项):`n$($items -join "`n")" }
    return ""
}

# ========== Prompt 构建 ==========
function Build-Prompt($phase, $projInfo) {
    $ctx = ""
    if ($script:requirement) { $ctx += "需求: $($script:requirement)`n" }
    if ($projInfo.stack -ne "未知") { $ctx += "技术栈: $($projInfo.stack)`n" }
    if ($projInfo.claude) { $ctx += "项目规范:`n$($projInfo.claude)`n" }

    switch ($phase) {
        1 { }
        2 {
            if (Test-Path "RESEARCH.md") { $ctx += "调研结果:`n$((Get-Content RESEARCH.md -Encoding UTF8 | Select-Object -First 20) -join "`n")`n" }
        }
        3 {
            if (Test-Path "PRD.md") { $ctx += "需求文档:`n$((Get-Content PRD.md -Encoding UTF8 | Select-Object -First 25) -join "`n")`n" }
        }
        4 {
            $ctx += Get-NextTask
            $ctx += "`n"
            $ctx += Get-GitStats
            # 编译检查：上次commit后是否能编译
            $build = Get-BuildResult
            if ($build) { $ctx += "`n$build`n" }
            $todo = Get-TodoItems
            if ($todo) { $ctx += "`n$todo`n" }
        }
        5 {
            $testResult = Get-TestResult
            if ($testResult) { $ctx += "`n$testResult`n" }
            $ctx += Get-GitStats
        }
        6 {
            if (Test-Path "TEST_REPORT.md") { $ctx += "测试报告:`n$((Get-Content TEST_REPORT.md -Encoding UTF8 | Select-Object -First 15) -join "`n")`n" }
            $ctx += Get-GitStats
        }
    }

    return "$base`n`n## 第${phase}阶段`n$ctx"
}

# ========== 主流程 ==========
[Console]::TreatControlCAsInput = $false

$proj = Detect-Project
if ($hasClaude) { $claudeStatus = "OK" } else { $claudeStatus = "N/A" }
if ($hasGit) { $gitStatus = "OK" } else { $gitStatus = "N/A" }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Auto Dev Agent" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Project: $($proj.name)" -ForegroundColor White
Write-Host " Stack:   $($proj.stack)" -ForegroundColor White
Write-Host " Dir:     $projDir" -ForegroundColor Gray
Write-Host " claude: $claudeStatus | git: $gitStatus" -ForegroundColor Gray
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$script:requirement = Resolve-Requirement $args[0] $proj
if (-not $script:requirement) { Write-Host "No requirement, exit" -ForegroundColor Yellow; exit }
$reqPreview = $script:requirement.Substring(0, [Math]::Min(80, $script:requirement.Length))
Log "Requirement: $reqPreview..."

$startPhase = Get-Phase
Log "Project: $($proj.name) | Stack: $($proj.stack) | Start phase: $startPhase"

$script:curJob = $null
$stuckCount = 0
$iteration = 0
$maxIterations = 3

try {
while ($iteration -lt $maxIterations) {
    $iteration++
    if ($iteration -eq 1) { $startP = $startPhase } else { $startP = 4 }
    Log "=== Iteration $iteration (start phase $startP) ==="

    for ($i = $startP; $i -le 6; $i++) {
        # 写入当前阶段到PROGRESS.md（确保Get-Phase读到正确值）
        if (-not (Test-Path "PROGRESS.md") -or $iteration -gt 1) {
            @"
当前阶段: $i
已完成: iteration $iteration
"@ | Out-File "PROGRESS.md" -Encoding UTF8
        }

        $phase = Get-Phase
        $prompt = Build-Prompt $phase $proj
        $snapshotBefore = Get-FileSnapshot
        $promptLen = $prompt.Length
        Log "Round $i -> stage $phase ($promptLen chars)"

        $script:curJob = Start-Job {
            param($p, $d)
            Set-Location $d
            claude --dangerously-skip-permissions --max-turns 150 --max-budget-usd 20 -p $p
        } -ArgumentList $prompt, $projDir

        $pStart = Get-Date
        $lastOutput = Get-Date
        $stopped = $false

        while (-not $stopped) {
            $elapsed = ((Get-Date) - $pStart).TotalSeconds
            $totalElapsed = ((Get-Date) - $totalStart).TotalSeconds
            $idleElapsed = ((Get-Date) - $lastOutput).TotalSeconds

            if ($totalElapsed -gt 2700) { Log "TOTAL TIMEOUT"; Cleanup; $stopped = $true; break }
            if ($elapsed -gt $phaseTimeout) { Log "PHASE TIMEOUT"; Cleanup; $stopped = $true; break }
            if ($idleElapsed -gt $idleTimeout) { Log "IDLE TIMEOUT"; Cleanup; $stopped = $true; break }

            if ($script:curJob.State -in "Completed","Failed") {
                Receive-Job $script:curJob | Select-Object -Last 5 | ForEach-Object { Write-Host $_ }
                break
            }

            if ($elapsed -gt 10 -and -not (Get-Process -Name "claude" -EA SilentlyContinue)) {
                Log "Claude process ended"; break
            }

            $out = Receive-Job $script:curJob -Keep 2>$null
            if ($out) { $lastOutput = Get-Date }
            Start-Sleep 5
        }

        Remove-Job $script:curJob -Force -EA SilentlyContinue
        $script:curJob = $null
        if ($stopped) { break 2 }

        $snapshotAfter = Get-FileSnapshot
        if (Check-Changed $snapshotBefore $snapshotAfter) {
            Update-Progress $phase
            $stuckCount = 0
            Log "Stage $phase has output, advancing"
        } else {
            $stuckCount++
            Log "No output detected (stuck: $stuckCount/2)"
            if ($stuckCount -ge 2) {
                Log "Stuck, forcing advance"
                Update-Progress $phase
                $stuckCount = 0
            }
        }

        Start-Sleep 3
    }

    # 检查是否还有待办（基于实际状态判断）
    $hasMore = $false

    # 1. 检查TODO.md未完成项
    if (Test-Path "TODO.md") {
        $pending = Get-Content TODO.md -Encoding UTF8 | Where-Object { $_ -match '^\s*[-*]\s*\[[ x]\]' -and $_ -notmatch '\[x\]' -and $_ -notmatch '\[X\]' }
        if ($pending.Count -gt 0) { $hasMore = $true; Log "TODO: $($pending.Count) items remaining" }
    }

    # 2. 检查IMPLEMENTATION_PLAN.md未完成任务
    if (-not $hasMore -and (Test-Path "IMPLEMENTATION_PLAN.md")) {
        $tasks = Get-Content IMPLEMENTATION_PLAN.md -Encoding UTF8 | Where-Object { $_ -match '^\s*[-*]\s*\[\s*\]' }
        if ($tasks.Count -gt 0) { $hasMore = $true; Log "Plan: $($tasks.Count) tasks remaining" }
    }

    # 3. 编译检查：代码能编译吗？
    if (-not $hasMore) {
        $build = Get-BuildResult
        if ($build -and $build -match "失败") { $hasMore = $true; Log "Build failed, need fix" }
    }

    # 4. 测试检查：测试全通过吗？
    if (-not $hasMore) {
        $test = Get-TestResult
        if ($test -and $test -notmatch "未检测到" -and ($test -match "FAILED|failed|失败|ERROR|error" -and $test -notmatch "0 failed|0 error")) {
            $hasMore = $true; Log "Tests failing, need fix"
        }
    }

    if (-not $hasMore) { Log "All clear, stopping"; break }
    Log "Issues found, starting iteration $($iteration + 1)"
}
} finally { Cleanup }

Log "All done"
Write-Host "`nDone!" -ForegroundColor Magenta
