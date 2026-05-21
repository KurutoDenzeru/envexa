import re
import subprocess
import shutil

_GEM_OUTDATED_RE = re.compile(r"^(\S+)\s+\((\S+)\s*(?:[<>]?\s*)?(\S*)\)")


def scan():
    result = {
        "tool": "gem",
        "available": False,
        "ruby_version": None,
        "outdated": [],
        "issues": [],
        "status": "ok",
    }

    if not shutil.which("ruby"):
        result["status"] = "skipped"
        result["issues"].append("Ruby not installed")
        return result

    try:
        rb_v = subprocess.run(
            ["ruby", "--version"], capture_output=True, text=True, timeout=5
        )
        result["ruby_version"] = rb_v.stdout.strip()

        if not shutil.which("gem"):
            return result

        outdated = subprocess.run(
            ["gem", "outdated"], capture_output=True, text=True, timeout=30
        )
        if outdated.returncode == 0 and outdated.stdout.strip():
            for line in outdated.stdout.strip().split("\n"):
                m = _GEM_OUTDATED_RE.match(line)
                if m:
                    result["outdated"].append(
                        {
                            "name": m.group(1),
                            "current": m.group(2),
                            "latest": m.group(3) if m.group(3) else "?",
                        }
                    )

        result["status"] = "ok" if not result["outdated"] else "warning"
        if result["outdated"]:
            result["issues"].append(f"{len(result['outdated'])} outdated gem(s)")
    except Exception as e:
        result["status"] = "error"
        result["issues"].append(str(e))

    return result
