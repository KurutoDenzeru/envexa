import subprocess
import shutil


def scan():
    result = {
        "tool": "deno",
        "available": False,
        "deno_version": None,
        "outdated": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("deno"):
        result["status"] = "skipped"
        result["issues"].append("deno not installed")
        return result

    try:
        deno_v = subprocess.run(
            ["deno", "--version"], capture_output=True, text=True, timeout=5
        )
        version_line = deno_v.stdout.strip().split("\n")[0] if deno_v.stdout else "unknown"
        result["deno_version"] = version_line
        result["available"] = True

        outdated = subprocess.run(
            ["deno", "outdated"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            for line in outdated.stdout.strip().split("\n"):
                parts = line.split()
                if len(parts) >= 3 and parts[0] not in ("Name", "Current", "Latest"):
                    result["outdated"].append(
                        {
                            "name": parts[0],
                            "current": parts[1],
                            "latest": parts[2],
                        }
                    )

        result["status"] = "ok" if not result["outdated"] else "warning"
        if result["outdated"]:
            result["issues"].append(
                f"{len(result['outdated'])} outdated package(s)"
            )
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
