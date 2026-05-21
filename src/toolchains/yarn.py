import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "yarn",
        "available": False,
        "node_version": None,
        "yarn_version": None,
        "outdated_global": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("yarn"):
        result["status"] = "skipped"
        result["issues"].append("yarn not installed")
        return result

    try:
        if shutil.which("node"):
            node_v = subprocess.run(
                ["node", "--version"], capture_output=True, text=True, timeout=5
            )
            result["node_version"] = node_v.stdout.strip()

        yarn_v = subprocess.run(
            ["yarn", "--version"], capture_output=True, text=True, timeout=5
        )
        result["yarn_version"] = yarn_v.stdout.strip()
        result["available"] = True

        outdated = subprocess.run(
            ["yarn", "outdated", "--json"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            lines = outdated.stdout.strip().split("\n")
            for line in lines:
                try:
                    data = json.loads(line)
                    if data.get("type") == "table":
                        for row in data.get("data", {}).get("body", []):
                            if len(row) >= 3:
                                result["outdated_global"].append(
                                    {
                                        "name": row[0],
                                        "current": row[1],
                                        "latest": row[2],
                                    }
                                )
                except json.JSONDecodeError:
                    continue

        result["status"] = "ok" if not result["outdated_global"] else "warning"
        if result["outdated_global"]:
            result["issues"].append(
                f"{len(result['outdated_global'])} outdated package(s)"
            )
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
