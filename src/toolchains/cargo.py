import subprocess
import shutil


def scan():
    result = {
        "tool": "cargo",
        "available": False,
        "rustc_version": None,
        "cargo_version": None,
        "outdated": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("rustc"):
        result["status"] = "skipped"
        result["issues"].append("Rust not installed")
        return result

    try:
        rs_v = subprocess.run(
            ["rustc", "--version"], capture_output=True, text=True, timeout=5
        )
        result["rustc_version"] = rs_v.stdout.strip()

        if not shutil.which("cargo"):
            return result

        cv = subprocess.run(
            ["cargo", "--version"], capture_output=True, text=True, timeout=5
        )
        result["cargo_version"] = cv.stdout.strip()

        installed = subprocess.run(
            ["cargo", "install", "--list"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        has_outdated = "cargo-outdated" in installed.stdout
        result["outdated_tool_available"] = has_outdated

        if not has_outdated:
            result["issues"].append(
                "cargo-outdated not installed (run: cargo install cargo-outdated)"
            )

        result["status"] = "ok"
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
