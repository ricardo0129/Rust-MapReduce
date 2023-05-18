cargo run --bin sequential data/pg-being_ernest.txt
pid=$!
cargo run --bin helloworld-server 3 data/pg-being_ernest.txt &
sleep 1
cargo run --bin worker &


wait $pid
cat mr-out-* | sort > mr-combined

if cmp mr-out mr-combined 
then
  echo "wc test: PASS"
else
  echo "failed"
fi




rm mr-*
