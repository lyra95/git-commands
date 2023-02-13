#!/usr/bin/env python
import re
import sys

class BCOLORS:
    GREEN = '\033[92m'
    RED = '\033[91m'
    ENDC = '\033[0m'


def try_get_key(line: str) -> int:
    m = re.search(r"#(?P<pr_num>\d+)\s", line)
    if not m:
        return -1
    return int(m.group('pr_num'))


def create(lines: list[str]) -> tuple[dict[int, str], list[str]]:
    result_dict = {}
    result_list = []
    for line in lines:
        key = try_get_key(line)
        if key == -1:
            result_list.append(line)
        else:
            result_dict[key] = line
    return result_dict, result_list


def main(cherrys: list[str], merges: list[str], no_color: bool):
    cherrys_dict, cherrys_irregulars = create(cherrys)
    merges_dict, merges_irregulars = create(merges)
    common = [cherrys_dict[key] for key in cherrys_dict.keys() if key in merges_dict]
    cherrys_only = {key: cherrys_dict[key] for key in cherrys_dict.keys() if key not in merges_dict}
    merges_only = {key: merges_dict[key] for key in merges_dict.keys() if key not in cherrys_dict}
    for line in cherrys_only.values():
        print(f"+ {line}" if no_color else f"{BCOLORS.GREEN}+ {line}{BCOLORS.ENDC}")
    for line in cherrys_irregulars:
        print(f"+? {line}" if no_color else f"{BCOLORS.GREEN}+? {line}{BCOLORS.ENDC}")
    for line in common:
        print(f"= {line}")
    for line in merges_irregulars:
        print(f"-? {line}" if no_color else f"{BCOLORS.RED}-? {line}{BCOLORS.ENDC}")
    for line in merges_only.values():
        print(f"- {line}" if no_color else f"{BCOLORS.RED}- {line}{BCOLORS.ENDC}")


if __name__ == "__main__":
    arg1: str = sys.argv[1]
    arg2: str = sys.argv[2]
    arg3: str = sys.argv[3] if len(sys.argv) > 3 else ""

    main(arg1.splitlines(), arg2.splitlines(), arg3 == "--no-color")
