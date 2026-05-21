import subprocess
import json
import shutil


def scan():
    result = {
        "tool": "docker",
        "available": False,
        "version": None,
        "disk_usage": {},
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("docker"):
        result["status"] = "skipped"
        result["issues"].append("Docker not installed")
        return result

    try:
        d_v = subprocess.run(
            ["docker", "version", "--format", "{{.Server.Version}}"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        result["version"] = d_v.stdout.strip() or "unknown"

        info = subprocess.run(
            ["docker", "info"], capture_output=True, text=True, timeout=10
        )
        if info.returncode != 0:
            result["status"] = "error"
            result["issues"].append("Docker daemon not running")
            return result

        df = subprocess.run(
            ["docker", "system", "df", "--format", "json"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if df.returncode == 0 and df.stdout.strip():
            for line in df.stdout.strip().split("\n"):
                if line.strip():
                    try:
                        data = json.loads(line)
                        typ = data.get("Type", "")
                        result["disk_usage"][typ] = {
                            "total": data.get("TotalCount", "?"),
                            "size": data.get("Size", "?"),
                            "reclaimable": data.get("ReclaimableSize", "?"),
                        }
                    except json.JSONDecodeError:
                        pass

        images = subprocess.run(
            ["docker", "images", "--filter", "dangling=true", "-q"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        dangling = [x for x in images.stdout.strip().split("\n") if x]
        if dangling:
            result["issues"].append(f"{len(dangling)} dangling image(s)")

        result["status"] = "ok"
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
