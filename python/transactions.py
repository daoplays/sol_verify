import solana.system_program as sp
from solana.publickey import PublicKey
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.rpc.types import TxOpts
import solana as sol
import subprocess
import base64
import numpy as np
import json
import base58
import time
import copy
import git
import os

from rpc_funcs import *
from state import *
from codes import *

def load_key(filename):
	skey = open(filename).readlines()[0][1:-1].split(",")
	int_key = []
	for element in skey:
		int_key.append(int(element))
		
	owner=sol.keypair.Keypair.from_secret_key(bytes(int_key)[:32])
	
	return owner

def load_config(filename):
    
    return json.load(open(filename))["config"]

def lsremote(url):

    remote_refs = {}

    g = git.cmd.Git()

    for ref in g.ls_remote(url).split('\n'):
        hash_ref_list = ref.split('\t')
        remote_refs[hash_ref_list[1]] = hash_ref_list[0]

    return remote_refs

def check_args(dev_client, user_pubkey, args):

    program_string = (base58.b58encode(bytearray(args.address))).decode("utf-8")

    if (not check_address_exists(dev_client, args.address)):
        update_idx = get_update_state_idx(user_pubkey, PROGRAM_DOESNT_EXIST, "Program " + program_string + " : address '" + program_string + "' does not exist or has no lamports")
        send_transaction(dev_client, [update_idx])
        return False

    try:
        remote = lsremote(args.git_repo)
    except:
        update_idx = get_update_state_idx(user_pubkey, GIT_REPO_DOESNT_EXIST, "Program " + program_string + " : git repo '" + args.git_repo + "' does not exist or inaccessible")
        send_transaction(dev_client, [update_idx])
        return False

    return remote


def write_config_file(args, user_pubkey, docker_count):

    program_string = (base58.b58encode(bytearray(args.address))).decode("utf-8")
    config_name = "verify_run_script_" + str(docker_count) + ".sh"

    f = open(config_name, "w")

    f.write("git clone " + args.git_repo + " test_repo > /root/logfile.txt\n")
    f.write("git clone https://github.com/daoplays/sol_verify.git >> /root/logfile.txt\n")

    f.write("cd /sol_verify/client\n")
    f.write("cargo run /root/.config/solana/id.json update_status " + user_pubkey + " 0 'Program " + program_string + " : sol_verify built, airdropping funds'\n")

    # to avoid rate limits create a new pubkey, airdrop to there and then transfer over
    f.write("solana-keygen new -o temp.json --no-bip39-passphrase\n")
    f.write("solana airdrop 2 temp.json\n")
    f.write("solana airdrop 2 temp.json\n")
    f.write("solana transfer --from temp.json /root/.config/solana/id.json 3.99\n")

    # check the provided directory exists
    f.write("[ ! -d \"/test_repo/" + args.directory + "\" ] && cd /sol_verify/client && cargo run /root/.config/solana/id.json update_status " + user_pubkey + " 102 \"Program " + program_string + " : directory " + args.directory + " doesn't exist in repo\" && exit 1\n")
  

    f.write("cargo run /root/.config/solana/id.json update_status " + user_pubkey + " 0 'Program " + program_string + " : cloning program repo and building program'\n")

    f.write("cd /test_repo\n")
    f.write("git checkout " + args.git_commit + "\n")
    f.write("cd " + args.directory + "\n")

    f.write("cargo build-bpf\n")
    f.write("solana program deploy target/deploy/*.so\n")
    f.write("cd /sol_verify/client\n")

    f.write("cargo run /root/.config/solana/id.json update_status " + user_pubkey + " 0 'Program " + program_string + " : running verification'\n")

    f.write("sleep 30\n")
    f.write("cargo run /root/.config/solana/id.json verify /test_repo/" +  args.directory + "/target/deploy/*-keypair.json " + program_string + "\n")


    f.write("cargo run /root/.config/solana/id.json update_status " + user_pubkey + " 1 'Program " + program_string + " : verification complete'\n")

    f.close()
    
    subprocess.run(["chmod", "777", config_name])

def check_for_finished_dockers(dev_client, dockers):

    new_dockers = copy.deepcopy(dockers)
    keys = dockers.keys()

    for key in keys:
        status_code = check_user_status_code(dev_client, key)
        if status_code == 1 or status_code >= 100:
            log_db("Found finished docker for " + key)
            subprocess.run(["../docker/stop.sh "+str(dockers[key])], shell=True)
            new_dockers.pop(key)

        time.sleep(1)

    return new_dockers

def check_address_exists(dev_client, address):

    try:
        response = dev_client.get_account_info(PublicKey(address))
    except :
        log_error("unable to get program account: " +  str(address))
        return False

    if "result" not in response.keys():
        log_error("result field not in json :")
        print(response)
        return False

    lamports = response["result"]["value"]["lamports"]

    if lamports > 0:
        return True
    
    return False

def check_meta_data_account(dev_client, program_address):
    config = load_config("config.json")
    
    meta_account, _user_bump = PublicKey.find_program_address([bytes(PublicKey(program_address))], PublicKey(PROGRAM_KEY))

    print("address ", meta_account)

    response = dev_client.get_account_info(meta_account)

    data = response["result"]["value"]["data"][0]
    decoded_data = base64.b64decode(data)

    return decoded_data

def check_user_status_code(dev_client, user_account_key):
    config = load_config("config.json")
    wallet = load_key(config["wallet"])
    
    user_account, _user_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(PROGRAM_KEY))

    log_db("Find program address for " + str(user_account_key) + " " +  str(user_account))
    
    try :

        response = dev_client.get_account_info(user_account)
    except :
        log_error("unable to get program account")
        return 0

    data = response["result"]["value"]["data"][0]
    decoded_data = base64.b64decode(data)
    byte_array = list(bytearray(decoded_data))
    status_code = byte_array[0]

    return status_code


def get_update_state_idx(user_account_key, status_code, log_message):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    user_account, _user_bump = PublicKey.find_program_address([bytes(PublicKey(user_account_key))], PublicKey(PROGRAM_KEY))

    status_code = np.uint8(status_code)
    #status_meta = StatusMeta.build({"address" : PublicKey(user_account_key), "status_code": status_code, "log_message" : log_message})

    instruction = TransactionInstruction(
        program_id = PublicKey(PROGRAM_KEY),
        data = Verifier_Instructions.build(Verifier_Instructions.enum.UpdateStatus(bytes(PublicKey(user_account_key)), status_code, log_message)),
        keys = [
            AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),
            AccountMeta(pubkey=user_account, is_signer=False, is_writable=True),
            AccountMeta(pubkey=sp.SYS_PROGRAM_ID, is_signer=False, is_writable=False)
            ]
    )

    return instruction


def send_transaction(dev_client, instructions):

    config = load_config("config.json")
    wallet = load_key(config["wallet"])

    blockhash = dev_client.get_recent_blockhash()['result']['value']['blockhash']
    txn = Transaction(recent_blockhash=blockhash, fee_payer=wallet.public_key)

    for idx in instructions:
        txn.add(idx)

    txn.sign(wallet)

    response = dev_client.send_transaction(
        txn,
        wallet,
        opts=TxOpts(skip_preflight=True, skip_confirmation=True)
    )

    print(response)