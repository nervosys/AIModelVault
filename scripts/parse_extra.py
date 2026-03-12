import json

with open("target/tarpaulin-report.json") as f:
    data = json.load(f)

targets = [
    "version_sqlite.rs", "rag/database.rs", "rag/mcp.rs",
    "crypto/compression.rs"
]

for finfo in data["files"]:
    path = "/".join(finfo["path"])
    if "src/" not in path:
        continue
    short = path.split("src/")[1] if "src/" in path else path
    if short not in targets:
        continue
    uncovered = [t["line"] for t in finfo["traces"] if t["stats"]["Line"] == 0]
    total = len(finfo["traces"])
    covered = total - len(uncovered)
    print(f"\n=== {short} ({covered}/{total}, {len(uncovered)} uncovered) ===")
    ranges = []
    for line in sorted(uncovered):
        if ranges and line == ranges[-1][1] + 1:
            ranges[-1] = (ranges[-1][0], line)
        else:
            ranges.append((line, line))
    for start, end in ranges:
        if start == end:
            print(f"  L{start}")
        else:
            print(f"  L{start}-L{end}")
