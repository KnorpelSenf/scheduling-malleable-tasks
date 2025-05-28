#!/bin/bash

set -ex

cd "$(dirname "$0")"

N=100
echo "ms,n,m,makespan" | tee eval-{ilp,lp,dp}.csv
for n in `seq 10 5 $N`
do 
    cargo run --release -- generate -n $n -m 4 --min 1 --max 10 -j jobs_basic_${n}.csv -o 8 --min-chain 1 --max-chain $n -c constraints_basic_${n}.csv --concave
    cargo run --release -- solve-ilp -j jobs_basic_${n}.csv -c constraints_basic_${n}.csv >> eval-ilp.csv
    cargo run --release -- solve-lp  -j jobs_basic_${n}.csv -c constraints_basic_${n}.csv >> eval-lp.csv
    cargo run --release -- solve-dp  -j jobs_basic_${n}.csv -c constraints_basic_${n}.csv >> eval-dp.csv
done
