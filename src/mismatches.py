import json
from pathlib import Path


def scan_projects(project_dirs: list[str] = None) -> dict:
    if not project_dirs:
        project_dirs = [str(Path.cwd())]

    results = {}

    for proj_dir in project_dirs:
        proj_path = Path(proj_dir).expanduser().resolve()
        if not proj_path.is_dir():
            continue

        deps = {}

        pkg_json = proj_path / "package.json"
        if pkg_json.exists():
            try:
                data = json.loads(pkg_json.read_text())
                deps["npm"] = {
                    **data.get("dependencies", {}),
                    **data.get("devDependencies", {}),
                }
            except (json.JSONDecodeError, OSError):
                pass

        pyproject = proj_path / "pyproject.toml"
        if pyproject.exists():
            content = pyproject.read_text()
            py_deps = {}
            in_block = False
            for line in content.split("\n"):
                stripped = line.strip()
                if stripped.startswith("[tool.poetry.dependencies]") or stripped.startswith("[project.dependencies]"):
                    in_block = True
                    continue
                if in_block:
                    if stripped.startswith("[") and not stripped.startswith("["):
                        break
                    if "=" in stripped and not stripped.startswith("#"):
                        parts = stripped.split("=", 1)
                        name = parts[0].strip()
                        ver = parts[1].strip().strip('"').strip("'")
                        if name and name not in ("python",):
                            py_deps[name] = ver
            if py_deps:
                deps["python"] = py_deps

        results[proj_path.name] = deps

    all_packages = {}
    for proj, toolchains in results.items():
        for tool, pkgs in toolchains.items():
            for name, version in pkgs.items():
                if name not in all_packages:
                    all_packages[name] = {}
                all_packages[name][proj] = version

    mismatches = {}
    for pkg, versions in all_packages.items():
        unique_versions = set(versions.values())
        if len(unique_versions) > 1:
            mismatches[pkg] = versions

    return mismatches
