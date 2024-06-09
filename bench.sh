

DIR="./test-formulas/bench"

EXE="./target/release/sat-solver"

TIMEOUT=5 # SECONDS
JOBS=4 # PHYSICAL CORES

rm sat-bench-tasks.txt

for ITEM in $DIR/*; do
  for ALGO in simple dpll cdcl; do
 
    echo "(timeout -k 0 $TIMEOUT sh -c \"/bin/time -f \"%e\" 2>&1 ./target/release/sat-solver --cnf $ITEM -a $ALGO >/dev/null\") | ./bench-line-format.sh $ITEM $ALGO \
      
    " >> sat-bench-tasks.txt

  done 
done

parallel -a sat-bench-tasks.txt -j $JOBS
