from solana.rpc.api import Client
import numpy as np
import time
import subprocess

from state import *
from rpc_funcs import *
from log import *
from transactions import *
from codes import *

# connect to solana node
quick_node_dev = "https://api.devnet.solana.com"

dev_client = Client(quick_node_dev)

if (not dev_client.is_connected()):
    log_error("cannot connect to quicknode endpoint.")
    exit()

current_slot = get_slot(dev_client)

dockers = {}
docker_count = 0

last_signature = None
while(True):

    dockers = check_for_finished_dockers(dev_client, dockers)
    signatures, last_signature = get_signatures(quick_node_dev, current_slot, np.inf, last_signature)

    if (len(signatures) > 0):
        transactions = get_transactions(quick_node_dev, signatures)


        for transaction in transactions:
            data = get_data_from_transaction(transaction["transaction"])

            for d in data:
                args = d["args"]
                user_pubkey = d["user"]
                program_string = (base58.b58encode(bytearray(args.address))).decode("utf-8")

                if (isinstance(args, Verifier_Instructions.enum.SubmitProgram)):

                    if (user_pubkey in dockers.keys()):
                        log_error("already running docker for user " + str(user_pubkey))
                        continue

                    # check that the args are valid
                    if (not check_args(dev_client, user_pubkey, args)):
                        log_error("invalid arguments:")
                        print_submit_meta(args)
                        continue


                    dockers[user_pubkey] = docker_count
                    print("have Submit:")
                    print_submit_meta(args)
                   
                    write_config_file(args, user_pubkey, docker_count)
                    if (not write_docker_file(dev_client, user_pubkey, args, docker_count)):
                        continue

                    update_idx = get_update_state_idx(user_pubkey, 0, "Program " + program_string + " : creating docker container")
                    send_transaction(dev_client, [update_idx])

                    try:
                        subprocess.run("../docker/build.sh " + str(docker_count), shell=True, check=True)
                    except:
                        log_error("build failed!")
                        update_idx = get_update_state_idx(user_pubkey, DOCKER_BUILD_FAILED, "Program " + program_string + " : docker build failed to complete, check version numbers are valid")
                        send_transaction(dev_client, [update_idx])
                        continue

                    update_idx = get_update_state_idx(user_pubkey, 0, "Program " + program_string + " : clone and build sol_verify repo")
                    send_transaction(dev_client, [update_idx])
                    time.sleep(5)
                    subprocess.run(["../docker/run.sh " + str(docker_count)], shell=True)
                    docker_count += 1
                   


        
    time.sleep(10)