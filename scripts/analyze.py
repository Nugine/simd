#!/usr/bin/python3
from dataclasses import dataclass
from typing import Any, Dict, List
import sys
import json

from tabulate import tabulate

BENCHES = [
    {"name": "base64-check", "metric": "throughput"},
    {"name": "base64-decode", "metric": "throughput"},
    {"name": "base64-encode", "metric": "throughput"},
    {"name": "base64-forgiving-decode", "metric": "throughput"},
    {"name": "hex-check", "metric": "throughput"},
    {"name": "hex-decode", "metric": "throughput"},
    {"name": "hex-encode", "metric": "throughput"},
    {"name": "base32-check", "metric": "throughput"},
    {"name": "base32-decode", "metric": "throughput"},
    {"name": "base32-encode", "metric": "throughput"},
    {"name": "uuid-format", "metric": "latency"},
    {"name": "uuid-parse", "metric": "latency"},
    {"name": "ascii-check", "metric": "throughput"},
]


@dataclass
class BenchResult:
    name: str
    metric: str
    functions: List[str]
    cases: List[str]
    data: Dict[str, Dict[str, float]]


def read_jsonl(path: str):
    with open(path) as f:
        for line in f.readlines():
            line = line.strip()
            if not line:
                continue
            yield json.loads(line)


def convert_criterion_jsonl(messages: List[Any]):
    for msg in messages:
        reason: str = msg["reason"]
        if reason != "benchmark-complete":
            continue

        parts = msg["id"].split("/")
        bench = parts[0]
        crate = parts[1]
        variant = parts[2]
        case = parts[3]

        time: float = msg["typical"]["estimate"]

        yield {
            "bench": bench,
            "crate": crate,
            "variant": variant,
            "case": case,
            "time": time,
        }


def append_if_not_exists(l, x):
    if x not in l:
        l.append(x)


def find(l, f):
    for x in l:
        if f(x):
            return x
    raise Exception()


def gather_results(items: List[Any]) -> List[BenchResult]:
    results: Dict[str, BenchResult] = {}

    for item in items:
        name = item["bench"]
        metric = find(BENCHES, lambda x: x["name"] == name)["metric"]
        r = results.setdefault(name, BenchResult(name, metric, [], [], {}))

        function = f'{item["crate"]}/{item["variant"]}'
        append_if_not_exists(r.functions, function)

        case = item["case"]
        append_if_not_exists(r.cases, case)

        time = item["time"]

        if metric == "throughput":
            count = int(case)
            throughput = count / time * 1e9 / (1 << 30)  # GiB/s
            data = throughput
        elif metric == "latency":
            data = time
        else:
            raise Exception()

        row = r.data.setdefault(function, {})
        row[case] = data

    return list(results.values())


def position(l, f):
    for i, x in enumerate(l):
        if f(x):
            return i
    raise Exception()


def render_markdown(results: List[BenchResult]):
    results.sort(key=lambda x: position(BENCHES, lambda y: y["name"] == x.name))

    for result in results:
        metric2unit = {"throughput": "GiB/s", "latency": "ns"}
        unit = metric2unit[result.metric]

        headers = [""] + result.cases

        table = []
        for function, data in result.data.items():
            row = [function]
            for case in result.cases:
                col = list(result.data[f][case] for f in result.functions)

                if result.metric == "throughput":
                    needs_bold = data[case] == max(col)
                elif result.metric == "latency":
                    needs_bold = data[case] == min(col)
                else:
                    raise Exception()

                if needs_bold:
                    cell = f"**{data[case]:5.3f}**"
                else:
                    cell = f"  {data[case]:5.3f}  "

                row.append(cell)

            table.append(row)

        print(f"#### {result.name} ({unit})")
        print()
        print(tabulate(table, headers, tablefmt="github"))
        print()


if __name__ == "__main__":
    path = sys.argv[1]
    messages = list(read_jsonl(path))
    items = list(convert_criterion_jsonl(messages))
    results = gather_results(items)
    render_markdown(results)
