import json
d = json.load(open('target/tarpaulin-report.json'))
files = d['files']
for f in sorted(files, key=lambda x: sum(1 for t in x['traces'] if t['stats']['Line'] == 0), reverse=True):
    fname = f['path']
    if 'src/' in fname:
        fname = fname[fname.rfind('src/'):]
    uncov = [t['line'] for t in f['traces'] if t['stats']['Line'] == 0]
    total = len(f['traces'])
    cov = total - len(uncov)
    if uncov:
        print(f'{fname}: {cov}/{total} ({100*cov/total:.1f}%) uncov_lines={uncov}')
