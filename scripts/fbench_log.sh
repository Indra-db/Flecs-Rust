#!/bin/bash

cargo criterion --plotting-backend disabled 2>&1 | tee ../flecs_ecs/benches/fbench_log/bench.log
./filter_bench_log.sh