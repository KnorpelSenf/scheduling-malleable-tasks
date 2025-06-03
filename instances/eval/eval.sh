#!/bin/bash

set -ex

cd "$(dirname "$0")"

JOB_FILE=./jobs.csv
CONSTRAINT_FILE=./constraints.csv

# m
# min
# max
# omega
# min-chain
# max-chain

echo "ms,n,m,makespan" | tee eval-{ilp,lp,dp}.csv
for n in `seq 10 5 5000`
do 
    cargo run -rq -- generate -j $JOB_FILE -c $CONSTRAINT_FILE -n $n -m 4 --min 1 --max 10 -o 8 --min-chain 1 --max-chain $n --concave
    ILP_OUTPUT=$(timeout 15 cargo run -rq -- solve-ilp -j $JOB_FILE -c $CONSTRAINT_FILE 2&>/dev/null)
    if [ $? == 124 ] 
    then
        break
    fi
    LP_OUTPUT=$(timeout 15 cargo run -rq -- solve-lp  -j $JOB_FILE -c $CONSTRAINT_FILE)
    if [ $? == 124 ] 
    then
        break
    fi
    DP_OUTPUT=$(timeout 15 cargo run -rq -- solve-dp  -j $JOB_FILE -c $CONSTRAINT_FILE)
    if [ $? == 124 ] 
    then
        break
    fi
    echo $ILP_OUTPUT >> eval-ilp.csv
    echo $LP_OUTPUT  >> eval-lp.csv
    echo $DP_OUTPUT  >> eval-dp.csv
done
