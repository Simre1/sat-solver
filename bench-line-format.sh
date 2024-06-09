FILE=$1
ALGO=$2

read -r TIME

if [ -z "$TIME" ]; then
  TIME=-1
fi

echo $FILE $ALGO $TIME

