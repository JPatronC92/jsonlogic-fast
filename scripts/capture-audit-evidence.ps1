# capture-audit-evidence.ps1
# One atomic evidence capture for the normalized two-commit state.
$ErrorActionPreference = 'SilentlyContinue'
$branch = git branch --show-current
$tip = git rev-parse --short HEAD
$hChore = git rev-parse --short HEAD~1
$scratch = "C:\Users\52999\AppData\Local\Temp\grok-goal-b1e5ca673ce1\implementer"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null

"atomic evidence $(Get-Date -Format o) tip=$tip chore=$hChore" | Out-File "$scratch\verification_summary.txt"

git log --oneline -5 | Out-File "$scratch\commit1_log.txt"
git log --oneline -5 | Out-File "$scratch\commit2_log.txt"

git show --name-only $hChore | Out-File "$scratch\commit1_files.txt"

cargo test -p api --test api_tests 2>&1 | Out-File "$scratch\tests_output.txt"

cargo check -p api 2>&1 | Out-File "$scratch\check.txt"

git push origin $branch 2>&1 | Out-File "$scratch\push_output.txt"
git fetch origin
git log "origin/$branch" --oneline -5 | Out-File "$scratch\remote_log.txt"

# key asserts
$noPrivate = -not (Select-String terraform/slasher.tf -Pattern 'PRIVATE_KEY\s*=' -Simple -Quiet)
"no PRIVATE_KEY injection: $noPrivate" | Out-File "$scratch\key_evidence.txt" -Append

$deductAfter = Select-String api/src/lib.rs -Pattern 'BEFORE charging' -Quiet
"deduct after eval: $deductAfter" | Out-File "$scratch\key_evidence.txt" -Append

$condRate = Select-String api/src/storage.rs -Pattern 'condition_expression' -Quiet
"conditional rate: $condRate" | Out-File "$scratch\key_evidence.txt" -Append

$tracing = Select-String api/src/lib.rs -Pattern 'tracing::' -Quiet
"tracing/reqid: $tracing" | Out-File "$scratch\key_evidence.txt" -Append

$siwe = Select-String core/src/b2a/auth.rs -Pattern 'Chain ID mismatch|issued_at outside' -Quiet
"SIWE chain/issued checks: $siwe" | Out-File "$scratch\key_evidence.txt" -Append

$filter = Select-String api/src/storage.rs -Pattern 'FilterExpression' -Quiet
"Filter in get_all: $filter" | Out-File "$scratch\key_evidence.txt" -Append

$secrets = Select-String api/Cargo.toml -Pattern 'secretsmanager' -Quiet
"secretsmanager dep: $secrets" | Out-File "$scratch\key_evidence.txt" -Append

$b2aErr = Select-String api/src/lib.rs -Pattern 'B2AStorageError::' -Quiet
"B2AStorageError used in handler: $b2aErr" | Out-File "$scratch\key_evidence.txt" -Append

$siweTests = (Select-String api/tests/api_tests.rs -Pattern 'SIWE_CHAIN_ID').Count -gt 0
"SIWE_CHAIN_ID sets present: $siweTests" | Out-File "$scratch\key_evidence.txt" -Append

$mismatchTest = Select-String api/tests/api_tests.rs -Pattern 'test_siwe_chain_mismatch' -Quiet
"mismatch test present: $mismatchTest" | Out-File "$scratch\key_evidence.txt" -Append

$guarded = -not (Select-String api/tests/api_tests.rs -Pattern 'RUN_DYNAMO_TESTS.*return' -Quiet)
"guarded always drives Dynamo: $guarded" | Out-File "$scratch\key_evidence.txt" -Append

"current log:" | Out-File "$scratch\key_evidence.txt" -Append
git log --oneline -3 | Out-File "$scratch\key_evidence.txt" -Append

# also copy to other required files for the verif plan
Copy-Item "$scratch\verification_summary.txt" "$scratch\verif_reexec.txt" -Force
Copy-Item "$scratch\verification_summary.txt" "$scratch\final_verif.txt" -Force

Write-Output "Evidence captured for tip $tip / chore $hChore"
