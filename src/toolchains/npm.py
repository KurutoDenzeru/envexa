import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "npm",
        "available": False,
        "node_version": None,
        "npm_version": None,
        "outdated_global": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("node"):
        result["status"] = "skipped"
        result["issues"].append("Node.js not installed")
        return result

    try:
        node_v = subprocess.run(
            ["node", "--version"], capture_output=True, text=True, timeout=5
        )
        result["node_version"] = node_v.stdout.strip()

        if shutil.which("npm"):
            npm_v = subprocess.run(
                ["npm", "--version"], capture_output=True, text=True, timeout=5
            )
            result["npm_version"] = npm_v.stdout.strip()

            outdated = subprocess.run(
                ["npm", "outdated", "-g", "--json"],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if outdated.returncode == 0 and outdated.stdout.strip():
                data = json.loads(outdated.stdout)
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
