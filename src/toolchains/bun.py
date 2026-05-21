import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "bun",
        "available": False,
        "bun_version": None,
        "outdated_global": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("bun"):
        result["status"] = "skipped"
        result["issues"].append("bun not installed")
        return result

    try:
        bun_v = subprocess.run(
            ["bun", "--version"], capture_output=True, text=True, timeout=5
        )
        result["bun_version"] = bun_v.stdout.strip()
        result["available"] = True

        outdated = subprocess.run(
            ["bun", "pm", "outdated", "--json"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            data = json.loads(outdated.stdout)
            if isinstance(data, list):
                result["outdated_global"] = [
                    {
                        "name": pkg.get("name", "?"),
                        "current": pkg.get("current", "?"),
                        "latest": pkg.get("latest", "?"),
                    }
                    for pkg in data
                ]
            elif isinstance(data, dict):
                result["outdated_global"] = [
                    {
                        "name": pkg,
                        "current": info.get("current", "?"),
                        "latest": info.get("latest", "?"),
                    }
                    for pkg, info in data.items()
                ]

        result["status"] = "ok" if not result["outdated_global"] else "warning"
        if result["outdated_global"]:
            result["issues"].append(
                f"{len(result['outdated_global'])} outdated package(s)"
            )
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
