#!/bin/bash

cd "$(dirname "$0")"

# currently no grid search for:
# omega
# min-chain
# max-chain

N=2000

# Define the function that encapsulates the logic for a single 'm' value
eval_for_m() {
    local m=$1 # m is passed as the first argument to the function

    # The rest of the original loop body for 'm'
    max=8
    INSTANCE_PATH=./dp-smol/instances
    RESULT_PATH=./dp-smol/results
    mkdir -p $INSTANCE_PATH $RESULT_PATH
    # ILP_FILE=$RESULT_PATH/result-ilp-m${m}-max${max}.csv
    # LP_FILE=$RESULT_PATH/result-lp-m${m}-max${max}.csv
    DP_FILE=$RESULT_PATH/result-dp-m${m}-max${max}.csv
    # N is accessible here because it's exported
    for n in `seq 10 10 $N`
    do
        cargo run -rq -- generate -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv -n $n -m $m --min 1 --max $max -o 8 --min-chain 1 --max-chain $n --concave
    done
    echo "ms,n,m,makespan" | tee $ILP_FILE $LP_FILE $DP_FILE
    for n in `seq 10 10 $N`
    do
        time timeout 15 cargo run -rq -- solve-dp -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv >> $DP_FILE
        if [ $? == 124 ]
        then
            break
        fi
    done
}

# Export the variable N so it's available in the subshells spawned by xargs
export N
# Export the function eval_for_m so xargs can call it via bash -c
export -f eval_for_m

# Generate the sequence for 'm' (2, 4, ..., 32)
# Pipe this sequence to xargs to run 'eval_for_m' in parallel
# -P 16: Run up to 16 processes in parallel
# -I {}: Replace {} with the input line (a value of m)
# bash -c "eval_for_m {}": Executes bash, which in turn calls our exported function
# with the value of m as its argument.
seq 2 2 32 | xargs -P 16 -I {} bash -c "eval_for_m {}"

echo "All parallel tasks for m launched."
