# Joins gungraun's JSON summaries of a base run and a head run, passed as $base and $head, into a
# markdown report of instruction counts.

def key: .module_path + "/" + (.id // "");
def name: .function_name + (if .id then "/" + .id else "" end);
def total(metric): .profiles[0].data.total.metrics[metric].values.new;
def commas: tostring | if length <= 3 then . else (.[:-3] | commas) + "," + .[-3:] end;
def cell: if . == null then "" else commas end;
def change(old; new): (new - old) / old * 100;
def percent: (. * 100 | round / 100) as $p
    | if $p > 0 then "+\($p)%" elif $p < 0 then "\($p)%" else "0%" end;
def threshold: 1;

if ($head | length) == 0 then
    "No benchmark results in gungraun's output.\n" | halt_error(1)
else
    ($base | map({key: key, value: .}) | from_entries) as $old
    | ($head | map(key)) as $head_keys
    | [
        ($head[] | . as $now | $old[key] as $was
            | if $was == null then
                {name: name, head: total("Ir"), status: "new"}
            else
                change($was | total("Ir"); total("Ir")) as $ir
                | {
                    name: name,
                    base: ($was | total("Ir")),
                    head: total("Ir"),
                    ir: $ir,
                    cycles: change($was | total("EstimatedCycles"); total("EstimatedCycles")),
                    status: (if $ir >= threshold then "regressed"
                        elif $ir <= -threshold then "improved"
                        else "within" end)
                }
            end),
        ($base[] | select(key as $k | $head_keys | any(. == $k) | not)
            | {name: name, base: total("Ir"), status: "removed"})
    ] as $rows
    | ($rows | group_by(.status) | map({key: .[0].status, value: length}) | from_entries) as $n
    | (["🔴 \($n.regressed // 0) regressed", "🟢 \($n.improved // 0) improved",
        "\($n.within // 0) within ±\(threshold)%"]
        + (if $n.new then ["\($n.new) new"] else [] end)
        + (if $n.removed then ["\($n.removed) removed"] else [] end)
        | join(" · ")),
    "",
    "| Benchmark | Base | Head | Instructions | Est. cycles |",
    "|:--|--:|--:|--:|--:|",
    ($rows[]
        | "| `\(.name)` | \(.base | cell) | \(.head | cell) | "
            + (if .status == "new" or .status == "removed" then .status
                else (.ir | percent) + (if .status == "regressed" then " 🔴"
                    elif .status == "improved" then " 🟢" else "" end) end)
            + " | \(if .cycles then (.cycles | percent) else "" end) |"),
    "",
    "_Base and Head are instruction counts from one run under Callgrind. The same code gives the same counts, so a change comes from the code. Marked at ±\(threshold)% of instructions._"
end
