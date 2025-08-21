#!/bin/bash

cargo criterion -p flecs_ecs --plotting-backend disabled 2>&1 | tee ${CARGO_MAKE_WORKING_DIRECTORY}/flecs_ecs/benches/fbench_log/bench.log
scripts/filter_bench_log.sh