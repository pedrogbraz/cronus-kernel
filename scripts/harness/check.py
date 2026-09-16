#!/usr/bin/env python3
"""CRONUS kernel checks for the development harness.

No extra Python packages. argv-only (the harness never uses a shell).

Tiers:
  quick     fmt + clippy::correctness + tests inferred from the git diff
  full      the CI contract (fmt, clippy correctness, cargo test --locked,
            no-default-features, release binary ≤ 12 MiB)
  critical  full + security-module tests (authz, session, HTTP, webhooks)

Worktrees always set CARGO_TARGET_DIR to this checkout's ./target so they
never execute another worktree's binary (see AGENTS.md).

A binary crate has no [lib]: `cargo test FILTER` matches test *names*.
Never pass two positional filters — cargo treats the second as an error.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAX_SCOPED_MODULES = 8
RELEASE_BUDGET = 12 * 1024 * 1024
FORCE_FULL = (
    "Cargo.toml",
    "Cargo.lock",
    "src/main.rs",
    "rust-toolchain.toml",
)
LANGUAGE_LOCK = (
    "LANGUAGE.md",
    "llms-full.txt",
    "src/cli/context_grammar.rs",
    "src/cli/mcp/error_codes.rs",
)
SECURITY_FILTER = (
    "api_security|http_dispatch|session::|access::|auth::|authz::|"
    "http_guard::|webhook::|files::|rate_limit::"
)


def die(message: str, code: int = 1) -> None:
    print("harness-check: " + message, file=sys.stderr)
    raise SystemExit(code)


def run(argv: list[str], timeout: int) -> None:
    print("+ " + " ".join(argv), flush=True)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(ROOT / "target")
    env["CARGO_TERM_COLOR"] = env.get("CARGO_TERM_COLOR", "always")
    try:
        completed = subprocess.run(
            argv,
            cwd=ROOT,
            env=env,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        die("timeout after {}s: {}".format(timeout, " ".join(argv)))
    except FileNotFoundError:
        die("comando ausente: " + argv[0], 2)
    if completed.returncode != 0:
        die("falhou ({})".format(completed.returncode))


def git(*args: str) -> tuple[int, str]:
    result = subprocess.run(
        ["git", "-C", str(ROOT), *args],
        capture_output=True,
        text=True,
        timeout=60,
    )
    return result.returncode, result.stdout


def changed_paths(base: str) -> list[str]:
    names = set()
    queries = (
        ["diff", "--name-only", "HEAD"],
        ["diff", "--name-only", "--cached"],
        ["ls-files", "--others", "--exclude-standard"],
        ["diff", "--name-only", "{}...HEAD".format(base)],
    )
    for args in queries:
        code, out = git(*args)
        if code != 0:
            continue
        for line in out.splitlines():
            line = line.strip()
            if line:
                names.add(line)
    return sorted(names)


def rust_module(path: str) -> str | None:
    if not path.startswith("src/") or not path.endswith(".rs"):
        return None
    rel = path[len("src/") : -len(".rs")]
    parts = rel.split("/")
    if parts[-1] == "mod":
        parts = parts[:-1]
    if not parts:
        return None
    return "::".join(parts)


def integration_target(path: str) -> str | None:
    if path.startswith("tests/") and path.endswith(".rs") and path.count("/") == 1:
        return Path(path).stem
    return None


def classify(paths: list[str]) -> dict:
    modules: list[str] = []
    integrations: list[str] = []
    language = False
    rustish = False
    force_full = False
    for path in paths:
        if path in FORCE_FULL:
            force_full = True
        if path in LANGUAGE_LOCK:
            language = True
        if path.endswith(".rs") or path in ("Cargo.toml", "Cargo.lock"):
            rustish = True
        module = rust_module(path)
        if module:
            modules.append(module)
        target = integration_target(path)
        if target:
            integrations.append(target)
            rustish = True
    # Unique, stable order.
    modules = sorted(set(modules))
    integrations = sorted(set(integrations))
    return {
        "modules": modules,
        "integrations": integrations,
        "language": language,
        "rustish": rustish,
        "force_full": force_full or len(modules) > MAX_SCOPED_MODULES,
    }


def cargo_test(filter_expr: str | None, extra: list[str] | None = None, timeout: int = 1800) -> None:
    # Binary crate: never `cargo test --lib`. One positional filter only.
    argv = ["cargo", "test", "--locked"]
    if extra:
        argv.extend(extra)
    if filter_expr:
        argv.append(filter_expr)
    run(argv, timeout)


def fmt() -> None:
    run(["cargo", "fmt", "--all", "--", "--check"], 300)


def clippy() -> None:
    run(
        ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "clippy::correctness"],
        1800,
    )


def no_default_features() -> None:
    run(
        ["cargo", "check", "--locked", "--no-default-features", "--all-targets"],
        1800,
    )


def release_budget() -> None:
    run(["cargo", "build", "--release", "--locked"], 3600)
    binary = ROOT / "target" / "release" / "cronus"
    if not binary.is_file():
        die("binário release ausente: " + str(binary))
    size = binary.stat().st_size
    print("release cronus: {} bytes (budget {})".format(size, RELEASE_BUDGET), flush=True)
    if size > RELEASE_BUDGET:
        die("binário release {} bytes acima do budget 12 MiB".format(size))


def quick(plan: dict) -> None:
    fmt()
    if plan["rustish"] or plan["force_full"]:
        clippy()
    if plan["force_full"]:
        cargo_test(None)
        return
    filters = [module + "::" for module in plan["modules"]]
    if plan["language"]:
        filters.append("context_grammar")
        filters.append("mcp_error_codes")
    if filters:
        cargo_test("|".join(filters), extra=["--bin", "cronus"])
    for target in plan["integrations"]:
        cargo_test(None, extra=["--test", target])


def full() -> None:
    fmt()
    clippy()
    cargo_test(None, timeout=3600)
    no_default_features()
    release_budget()


def critical() -> None:
    full()
    cargo_test(SECURITY_FILTER, extra=["--bin", "cronus"])


def self_test() -> None:
    assert rust_module("src/session.rs") == "session"
    assert rust_module("src/parser/mod.rs") == "parser"
    assert rust_module("src/cli/mcp/error_codes.rs") == "cli::mcp::error_codes"
    assert rust_module("src/main.rs") == "main"
    assert integration_target("tests/mcp_cli.rs") == "mcp_cli"
    assert integration_target("tests/mcp/tools.rs") is None
    plan = classify(
        [
            "src/session.rs",
            "LANGUAGE.md",
            "tests/build_cli.rs",
        ]
    )
    assert plan["modules"] == ["session"]
    assert plan["language"] is True
    assert plan["integrations"] == ["build_cli"]
    assert plan["force_full"] is False
    forced = classify(["Cargo.lock", "src/access.rs"])
    assert forced["force_full"] is True
    print("self-test ok")


def main(argv: list[str]) -> int:
    if not (ROOT / "Cargo.toml").is_file():
        die("não parece o cronus-kernel (Cargo.toml ausente)", 2)
    os.chdir(ROOT)
    if argv == ["--self-test"]:
        self_test()
        return 0
    if len(argv) != 1 or argv[0] not in ("quick", "full", "critical"):
        die("uso: check.py quick|full|critical", 2)
    tier = argv[0]
    base = "main"
    project = ROOT / ".harness" / "project.json"
    if project.is_file():
        try:
            base = json.loads(project.read_text()).get("base_branch") or "main"
        except json.JSONDecodeError:
            pass
    plan = classify(changed_paths(base))
    print(
        "tier={} rustish={} language={} force_full={} modules={} integrations={}".format(
            tier,
            plan["rustish"],
            plan["language"],
            plan["force_full"],
            ",".join(plan["modules"]) or "-",
            ",".join(plan["integrations"]) or "-",
        ),
        flush=True,
    )
    if tier == "quick":
        quick(plan)
    elif tier == "full":
        full()
    else:
        critical()
    print("harness-check: {} ok".format(tier), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
