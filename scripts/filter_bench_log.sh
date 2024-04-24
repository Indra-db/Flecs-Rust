awk '{
    if (/time:/) {
        gsub(/^[ \t]+|[ \t]+$/, "", prev);
        gsub(/[ \t]+/, " ", $0);
        split($0, a, " ");
        if (a[1] != "time:") {        
            title = a[1];          
            unit = a[4];             
            timing = a[5];          
        } else {                   
            title = prev;          
            unit = a[3];
            timing = a[4];         
        }
        title_width = 80;            
        padding = title_width - length(title);
        printf "%-*s %s %s\n", title_width, title, timing, unit;
    }
    prev=$0
}' ../flecs_ecs/benches/fbench_log/bench.log > ../flecs_ecs/benches/fbench_log/bench_filtered.log
