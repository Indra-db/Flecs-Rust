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
        title_width = 50;            
        padding = title_width - length(title);
        printf "%-*s %s %s\n", title_width, title, timing, unit;
    }
    prev=$0
}' ${CARGO_MAKE_WORKING_DIRECTORY}/flecs_ecs/benches/fbench_log/bench.log > ${CARGO_MAKE_WORKING_DIRECTORY}/flecs_ecs/benches/fbench_log/bench_filtered.log
