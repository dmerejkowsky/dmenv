import sys
import subprocess

import requests
import toml

from installer import get_url


def main():
    check_git_status()
    version = get_latest_version()
    print("Bumping installer to", version)
    check_github_deliveries(version)
    patch_installer(version)
    commit()


def check_git_status():
    current_branch = run_git_captured("rev-parse", "--abbrev-ref", "HEAD")
    if current_branch != "master":
        sys.exit("not on master")
    status = run_git_captured("status", "--porcelain")
    if status:
        # sys.exit(status)
        pass

    behind = run_git_captured("rev-list", "HEAD..@{upstream}")
    if behind:
        sys.exit("Behind upstream")


def check_github_deliveries(version):
    for platform in ["linux", "darwin", "windows"]:
        url = get_url(platform, version)
        print("Checking", url, end="... ")
        response = requests.head(url)
        if not response.ok:
            print(response.status_code)
            sys.exit(1)
        print("ok")


def get_latest_version():
    with open("tbump.toml") as stream:
        parsed = toml.load(stream)
        return "v" + parsed["version"]["current"]


def patch_installer(version):
    with open("installer.py", "r") as stream:
        old_lines = stream.readlines()

    new_lines = []
    for line in old_lines:
        if line.startswith("VERSION ="):
            new_lines.append('VERSION = "%s"' % version)
        else:
            new_lines.append(line)

    with open("installer.py", "w") as stream:
        stream.writelines(new_lines)


def commit():
    pass


def get_current_branch():
    return run_git_captured("rev-parse", "--abbrev-ref", "HEAD")


def run_git_captured(*cmd):
    git_cmd = list(cmd)
    git_cmd.insert(0, "git")
    process = subprocess.run(
        git_cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True
    )
    return process.stdout.decode().strip("\n")


if __name__ == "__main__":
    main()
