#!/usr/bin/env python3
"""Aegis synthetic detection corpora generator (story S0.4). Stdlib only.

    python -I tests/corpora/generate.py           # (re)write data/*.jsonl + MANIFEST.json
    python -I tests/corpora/generate.py --check   # regenerate in memory, compare with disk, validate

Deterministic: each corpus uses its own random.Random seeded from a fixed string
(see cgen_common.SEED_PREFIX). All values are synthetic; see README.md.
"""
import hashlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)  # needed under `python -I`, which does not add the script dir

from cgen_common import ACTIONS, CATEGORIES, SEED_PREFIX, SEVERITIES, VERSION  # noqa: E402
from cgen_defence import gen_defence  # noqa: E402
from cgen_injection import gen_injection  # noqa: E402
from cgen_obfuscation import gen_obfuscation  # noqa: E402
from cgen_pii import gen_other_cats, gen_pii_neg, gen_pii_pos  # noqa: E402
from cgen_preservation import gen_preservation  # noqa: E402
from cgen_secrets import gen_secrets_neg, gen_secrets_pos  # noqa: E402

DATA = os.path.join(HERE, "data")
GENERATORS = [gen_secrets_pos, gen_secrets_neg, gen_pii_pos, gen_pii_neg, gen_other_cats,
              gen_obfuscation, gen_injection, gen_defence, gen_preservation]
SOURCE_FILES = ["generate.py", "cgen_common.py", "cgen_secrets.py", "cgen_pii.py", "cgen_obfuscation.py",
                "cgen_injection.py", "cgen_defence.py", "cgen_preservation.py"]


def dumps(row):
    return json.dumps(row, ensure_ascii=True, separators=(",", ":"))


def validate(corpus):
    seen = set()
    for row in corpus.rows:
        assert row["id"] not in seen, row["id"]
        seen.add(row["id"])
        assert row["expected_action"] in ACTIONS, row["id"]
        tb = row["text"].encode("utf-8")
        for lab in row["labels"]:
            assert lab["category"] in CATEGORIES and lab["severity"] in SEVERITIES, row["id"]
            s, e = lab["start"], lab["end"]
            assert 0 <= s < e <= len(tb), (row["id"], s, e)
            tb[s:e].decode("utf-8")  # must be on UTF-8 boundaries
            assert row["text"][lab["char_start"]:lab["char_end"]].encode("utf-8") == tb[s:e], row["id"]
        if "parts" in row:
            assert "\n".join(p["text"] for p in row["parts"]) == row["text"], row["id"]
        if row["expected_action"] in ("review", "block") and not row["labels"]:
            raise AssertionError(f"{row['id']}: review/block without labels")


def build():
    out = {}
    for g in GENERATORS:
        corpus = g()
        validate(corpus)
        out[corpus.name] = corpus
    return out


def summarise(corpora):
    counts = {}
    for name, c in corpora.items():
        cats, subs, acts = {}, {}, {}
        for row in c.rows:
            acts[row["expected_action"]] = acts.get(row["expected_action"], 0) + 1
            for lab in row["labels"]:
                cats[lab["category"]] = cats.get(lab["category"], 0) + 1
                subs[lab["subtype"]] = subs.get(lab["subtype"], 0) + 1
        counts[name] = {"records": len(c.rows),
                        "negatives": sum(1 for r in c.rows if not r["labels"]),
                        "labels_by_category": dict(sorted(cats.items())),
                        "labels_by_subtype": dict(sorted(subs.items())),
                        "records_by_expected_action": dict(sorted(acts.items()))}
    return counts


def file_bytes(corpus):
    return ("\n".join(dumps(r) for r in corpus.rows) + "\n").encode("utf-8")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def manifest_bytes(corpora, blobs):
    gen_hash = hashlib.sha256()
    for f in SOURCE_FILES:
        with open(os.path.join(HERE, f), "rb") as fh:
            gen_hash.update(fh.read().replace(b"\r\n", b"\n"))
    m = {"corpus_version": VERSION, "seed_prefix": SEED_PREFIX, "generator_sources_sha256": gen_hash.hexdigest(),
         "offset_unit": "utf8_bytes_end_exclusive",
         "files": {f"data/{n}.jsonl": {"sha256": sha(blobs[n]), "records": len(corpora[n].rows)} for n in corpora},
         "counts": summarise(corpora)}
    return (json.dumps(m, indent=2, sort_keys=True) + "\n").encode("utf-8")


def main():
    check = "--check" in sys.argv
    corpora = build()
    blobs = {n: file_bytes(c) for n, c in corpora.items()}
    mb = manifest_bytes(corpora, blobs)
    targets = {os.path.join(DATA, f"{n}.jsonl"): b for n, b in blobs.items()}
    targets[os.path.join(HERE, "MANIFEST.json")] = mb
    if check:
        bad = []
        for path, b in targets.items():
            try:
                with open(path, "rb") as fh:
                    if fh.read().replace(b"\r\n", b"\n") != b:
                        bad.append(path)
            except FileNotFoundError:
                bad.append(path)
        if bad:
            print("MISMATCH:", *bad, sep="\n  ")
            return 1
        print("OK: on-disk corpora match regeneration; all validation checks passed")
        return 0
    os.makedirs(DATA, exist_ok=True)
    for path, b in targets.items():
        with open(path, "wb") as fh:
            fh.write(b)
    total = sum(len(c.rows) for c in corpora.values())
    for n, c in corpora.items():
        print(f"{n:22s} {len(c.rows):4d}")
    print(f"{'TOTAL':22s} {total:4d}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
