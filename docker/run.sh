docker run --name docker_verify_$1 --rm -t -d daoplays/verify_$1
docker cp /root/.config/solana/id.json docker_verify_$1:/root/.config/solana/id.json
docker cp /home/daoplays/sol_verify/python/verify_run_script_$1.sh docker_verify_$1:/verify_run_script.sh 
docker exec -d docker_verify_$1 bash -c "./verify_run_script.sh"
rm verify_run_script_$1.sh

