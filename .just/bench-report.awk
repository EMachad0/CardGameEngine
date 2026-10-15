# Turns criterion's output from a base run and a head run, given in that order, into a markdown
# report with criterion's verdict for each benchmark.

FNR == 1 { id = ""; in_change = 0 }

{ run = FILENAME == ARGV[1] ? "base" : "head" }

/^Benchmarking [^:]+$/ {
    id = substr($0, 14)
    if (!(id in seen)) {
        seen[id] = 1
        order[++count] = id
    }
    next
}

/^ *change:$/ { in_change = 1; next }

/(change|time): +\[/ {
    split(bracketed($0), f, " ")
    if (in_change || $0 ~ /change:/) {
        change[id] = f[2]
        in_change = 0
    } else {
        time[run, id] = f[3] " " f[4]
    }
    next
}

/Performance has regressed\./ { verdict[id] = "regressed" }
/Performance has improved\./ { verdict[id] = "improved" }
/Change within noise threshold\./ { verdict[id] = "noise" }
/No change in performance detected\./ { verdict[id] = "none" }

function bracketed(line) {
    sub(/^[^[]*\[/, "", line)
    sub(/\].*$/, "", line)
    return line
}

function percent(s) {
    gsub(/−/, "-", s)
    sub(/%$/, "", s)
    if (s + 0 > -0.05 && s + 0 < 0.05) return "0.0%"
    return sprintf("%+.1f%%", s + 0)
}

END {
    for (i = 1; i <= count; i++) if (("head", order[i]) in time) head_results++
    if (!head_results) {
        print "No benchmark results found in criterion's output." > "/dev/stderr"
        exit 1
    }

    label["regressed"] = "🔴 Regressed"
    label["improved"] = "🟢 Improved"
    label["noise"] = "Within noise"
    label["none"] = "No change"
    label["new"] = "New"
    label["removed"] = "Removed"

    for (i = 1; i <= count; i++) {
        id = order[i]
        if (!(("head", id) in time)) status[id] = "removed"
        else if (!(id in verdict)) status[id] = "new"
        else status[id] = verdict[id]
        tally[status[id]]++
    }

    summary = "🔴 " (tally["regressed"] + 0) " regressed · 🟢 " (tally["improved"] + 0) " improved · " \
        (tally["noise"] + tally["none"]) " unchanged"
    if (tally["new"]) summary = summary " · " tally["new"] " new"
    if (tally["removed"]) summary = summary " · " tally["removed"] " removed"
    print summary
    print ""
    print "| Benchmark | Base | Head | Change | Verdict |"
    print "|:--|--:|--:|--:|:--|"
    for (i = 1; i <= count; i++) {
        id = order[i]
        delta = id in change ? percent(change[id]) : ""
        printf "| `%s` | %s | %s | %s | %s |\n", id, time["base", id], time["head", id], delta, label[status[id]]
    }
    print ""
    print "_Change is criterion's estimate of the change in mean time, so it can differ from Head over Base._"
}
