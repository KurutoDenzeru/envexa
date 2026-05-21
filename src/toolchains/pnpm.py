import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "pnpm",
        "available": False,
        "node_version": None,
        "pnpm_version": None,
        "outdated_global": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("pnpm"):
        result["status"] = "skipped"
        result["issues"].append("pnpm not installed")
        return result

    try:
        if shutil.which("node"):
            node_v = subprocess.run(
                ["node", "--version"], capture_output=True, text=True, timeout=5
            )
            result["node_version"] = node_v.stdout.strip()

        pnpm_v = subprocess.run(
            ["pnpm", "--version"], capture_output=True, text=True, timeout=5
        )
        result["pnpm_version"] = pnpm_v.stdout.strip()
        result["available"] = True

        outdated = subprocess.run(
            ["pnpm", "outdated", "--global", "--json"],
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
                f"{len(result['outdated_global'])} outdated global package(s)"
            )
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
