#!/bin/bash

cd "$(dirname "$0")"

# currently no grid search for:
# omega
# min-chain
# max-chain

N=200

for m in `seq 2 2 32`
do
    for max in {2,3,4,5,10,20}
    do
        echo "+++++++++++++++++ m $m max $max ++++++++++++++++++"
        INSTANCE_PATH=/results/instances
        RESULT_PATH=/results/results
        mkdir -p $INSTANCE_PATH $RESULT_PATH
        ILP_FILE=$RESULT_PATH/result-ilp-m${m}-max${max}.csv
        LP_FILE=$RESULT_PATH/result-lp-m${m}-max${max}.csv
        DP_FILE=$RESULT_PATH/result-dp-m${m}-max${max}.csv
        for n in `seq 10 5 $N`
        do 
            cargo run -rq -- generate -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv -n $n -m $m --min 1 --max $max -o 8 --min-chain 1 --max-chain $n --concave
        done
        echo "ms,n,m,makespan" | tee $ILP_FILE $LP_FILE $DP_FILE
        for n in `seq 10 5 $N`
        do 
            time timeout 15 cargo run -rq -- solve-ilp -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv >> $ILP_FILE
            if [ $? == 124 ] 
            then
                break
            fi
        done
        for n in `seq 10 5 $N`
        do 
            time timeout 15 cargo run -rq -- solve-lp -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv >> $LP_FILE
            if [ $? == 124 ] 
            then
                break
            fi
        done
        for n in `seq 10 5 $N`
        do 
            time timeout 15 cargo run -rq -- solve-dp -j $INSTANCE_PATH/jobs-n${n}-m${m}.csv -c $INSTANCE_PATH/constraints-n${n}-m${m}.csv >> $DP_FILE
            if [ $? == 124 ] 
            then
                break
            fi
        done
        echo "----------------- m $m max $max ------------------"
    done
done
