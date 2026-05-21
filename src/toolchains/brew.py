import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "brew",
        "available": False,
        "version": None,
        "outdated_formulae": [],
        "outdated_casks": [],
        "installed_count": 0,
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("brew"):
        result["status"] = "skipped"
        result["issues"].append("Homebrew not installed")
        return result

    try:
        ver = subprocess.run(
            ["brew", "--version"], capture_output=True, text=True, timeout=10
        )
        result["version"] = (
            ver.stdout.strip().split()[1] if ver.stdout else "unknown"
        )

        outdated = subprocess.run(
            ["brew", "outdated", "--json"], capture_output=True, text=True, timeout=30
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            data = json.loads(outdated.stdout)
            result["outdated_formulae"] = [
                {
                    "name": f["name"],
                    "current": f.get("installed_versions", ["?"])[0],
                    "latest": f.get("current_version", "?"),
                }
                for f in data.get("formulae", [])
            ]

        cask_outdated = subprocess.run(
            ["brew", "outdated", "--cask", "--greedy", "--json"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if cask_outdated.returncode == 0 and cask_outdated.stdout.strip():
            data = json.loads(cask_outdated.stdout)
            result["outdated_casks"] = [
                {
                    "name": c["name"],
                    "current": c.get("installed_versions", ["?"])[0],
                    "latest": c.get("current_version", "?"),
                }
                for c in data.get("casks", [])
            ]

        count = subprocess.run(
            ["brew", "list", "--formula", "--versions"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if count.returncode == 0:
            result["installed_count"] = len(
                [l for l in count.stdout.strip().split("\n") if l]
            )

        total_outdated = len(result["outdated_formulae"]) + len(
            result["outdated_casks"]
        )
        result["status"] = "ok" if total_outdated == 0 else "warning"
        if total_outdated > 0:
            result["issues"].append(f"{total_outdated} outdated package(s)")
    except subprocess.TimeoutExpired:
        result["status"] = "error"
        result["issues"].append("brew timed out")
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
