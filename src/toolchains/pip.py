import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "pip",
        "available": False,
        "python_version": None,
        "outdated": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("python3"):
        result["status"] = "skipped"
        result["issues"].append("Python 3 not installed")
        return result

    try:
        py_v = subprocess.run(
            ["python3", "--version"], capture_output=True, text=True, timeout=5
        )
        result["python_version"] = py_v.stdout.strip()

        if not shutil.which("pip3"):
            return result

        outdated = subprocess.run(
            ["pip3", "list", "--outdated", "--format=json"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            data = json.loads(outdated.stdout)
            result["outdated"] = [
                {
                    "name": pkg["name"],
                    "current": pkg.get("version", "?"),
                    "latest": pkg.get("latest_version", "?"),
                }
                for pkg in data
            ]

        result["status"] = "ok" if not result["outdated"] else "warning"
        if result["outdated"]:
            result["issues"].append(
                f"{len(result['outdated'])} outdated pip package(s)"
            )
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
