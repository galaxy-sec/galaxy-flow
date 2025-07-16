echo  "hello"

# Check if OUT_FILE environment variable exists and write "DATA" to it if it does
if [ -n "$OUT_FILE" ]; then
    echo "DATA" >> $OUT_FILE
fi

if [ -n "$NAME" ]; then
    echo $NAME >> $OUT_FILE

fi
