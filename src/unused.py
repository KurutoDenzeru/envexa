import subprocess
import json
from pathlib import Path


def check_unused(project_dir: str) -> dict:
    proj_path = Path(project_dir).expanduser().resolve()
    results = {}

    if (proj_path / "package.json").exists():
        try:
            result = subprocess.run(
                ["npx", "-y", "depcheck", "--json"],
                capture_output=True,
                text=True,
                timeout=60,
                cwd=str(proj_path),
            )
            if result.returncode == 0 and result.stdout.strip():
                data = json.loads(result.stdout)
                results["npm"] = {
                    "unused": data.get("dependencies", []),
                    "devUnused": data.get("devDependencies", []),
                    "missing": list(data.get("missing", {}).keys()),
                }
        except (subprocess.TimeoutExpired, FileNotFoundError, json.JSONDecodeError):
            pass

    return results
