import json
import sys

with open("target/tarpaulin-report.json") as f:
    data = json.load(f)

results = []
for finfo in data["files"]:
    path = "/".join(finfo["path"])
    if "src/" not in path:
        continue
    total = len(finfo["traces"])
    covered = sum(1 for t in finfo["traces"] if t["stats"]["Line"] > 0)
    uncov = total - covered
    pct = (covered / total * 100) if total > 0 else 0
    results.append((path, covered, total, uncov, pct))

results.sort(key=lambda x: -x[3])

print(f"{'File':<45} {'Cov':>5} {'Tot':>5} {'Unc':>5} {'Pct':>6}")
print("-" * 72)
for path, cov, tot, unc, pct in results:
    short = path.split("src/")[1] if "src/" in path else path
    print(f"{short:<45} {cov:>5} {tot:>5} {unc:>5} {pct:>5.1f}%")

total_cov = sum(r[1] for r in results)
total_tot = sum(r[2] for r in results)
total_unc = sum(r[3] for r in results)
total_pct = total_cov / total_tot * 100 if total_tot > 0 else 0
print(f"\n{'TOTAL':<45} {total_cov:>5} {total_tot:>5} {total_unc:>5} {total_pct:>5.1f}%")
