#!/bin/bash

# base_dir is the root of the habitat project
base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# First make sure that we have services already compiled to test.
pushd ../../
# make build-srv || exit $?
# cd components/dredd-rpc && cargo build
popd
cd /tmp

name=$(date | md5sum | awk '{ print $1  }')
dir="/tmp/$name"
key_dir="$dir/key-dir"
mkdir -p $dir $key_dir

# This will produce a URI that looks like
# postgresql://hab@127.0.0.1:39605/test
pg=$(su -c "pg_tmp -t -w 120 -o \"-c max_locks_per_transaction=128\"" hab)
port=$(echo "$pg" | awk -F ":" '{ print $3 }' | awk -F "/" '{ print $1 }')

# Write out some config files
cat << EOF > $dir/config_api.toml
[depot]
builds_enabled = true
non_core_builds_enabled = true
key_dir = "$key_dir"

[github]
app_private_key = "$key_dir/builder-github-app.pem"

[segment]
write_key = "hahano"
EOF

cat << EOF > $dir/config_jobsrv.toml
key_dir = "$key_dir"

[archive]
backend = "local"
local_dir = "/tmp"

[datastore]
host = "127.0.0.1"
port = $port
user = "hab"
database = "test"
connection_retry_ms = 300
connection_timeout_sec = 3600
connection_test = false
pool_size = 1
EOF

cat << EOF > $dir/config_sessionsrv.toml
[permissions]
admin_team = 1995301
build_worker_teams = [1995301]
early_access_teams = [1995301]

[github]
app_private_key = "$key_dir/builder-github-app.pem"

[datastore]
host = "127.0.0.1"
port = $port
user = "hab"
database = "test"
connection_retry_ms = 300
connection_timeout_sec = 3600
connection_test = false
pool_size = 1
EOF

cat << EOF > $dir/config_worker.toml
auth_token = "hahano"
bldr_url = "http://localhost:9636"
auto_publish = true
data_path = "/tmp"

[github]
app_private_key = "$key_dir/builder-github-app.pem"
EOF

cat << EOF > $dir/config_originsrv.toml
[datastore]
host = "127.0.0.1"
port = $port
user = "hab"
database = "test"
connection_retry_ms = 300
connection_timeout_sec = 3600
connection_test = false
pool_size = 1
EOF

cat << EOF > $dir/Procfile
api: $base_dir/target/debug/bldr-api start --path $dir/depot --config $dir/config_api.toml
router: $base_dir/target/debug/bldr-router start
jobsrv: $base_dir/target/debug/bldr-jobsrv start --config $dir/config_jobsrv.toml
sessionsrv: $base_dir/target/debug/bldr-sessionsrv start --config $dir/config_sessionsrv.toml
originsrv: $base_dir/target/debug/bldr-originsrv start --config $dir/config_originsrv.toml
worker: $base_dir/target/debug/bldr-worker start --config $dir/config_worker.toml
EOF

# Probably need to generate a box key pair at some point

# Start all the services up
$base_dir/support/linux/bin/forego start -f "$dir/Procfile" -e "$base_dir/support/bldr.env" 2>&1 > "$dir/services.log" &
forego_pid=$!

echo "**** Spinning up the services ****"
total=0
originsrv=0
sessionsrv=0
router=0
api=0
jobsrv=0
worker=0

while [ $total -ne 6 ]; do
  for svc in originsrv sessionsrv router api jobsrv worker; do
    if grep -q "builder-$svc is ready to go" "$dir/services.log"; then
      declare "$svc=1"
    fi
  done

  total=$(($originsrv + $sessionsrv + $router + $api + $jobsrv + $worker))
  sleep 1
done
echo "**** All services ready ****"

# Run the tests
dredd "$base_dir/components/builder-depot/doc/api.apib" http://localhost:9636 --language=rust --hookfiles="$base_dir/target/debug/dredd-rpc"
dredd_exit_code=$?
echo "**** Stopping services ****"
kill -INT $forego_pid
rm -fr $dir
exit $dredd_exit_code
