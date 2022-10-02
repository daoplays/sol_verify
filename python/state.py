from borsh_construct import Enum, CStruct, String, U64, U8
import base58
from solana import message

PROGRAM_KEY = "CNd6wN4en9Xvbf2e1ubb2YyCkC7J1BbbuhAGhqcdHFbi"

def print_submit_meta(meta):
    print("address: ", base58.b58encode(bytearray(meta.address)).decode("utf-8"))
    print("network: ", meta.network)
    print("git_repo: ", meta.git_repo)
    print("git_commit: ", meta.git_commit)
    print("directory: ", meta.directory)
    print("docker_version: ", meta.docker_version)
    print("rust_version: ", meta.rust_version)
    print("solana_version: ", meta.solana_version)
    print("anchor_version: ", meta.anchor_version)

Verifier_Network = Enum(
    "test_net",
    "dev_net",
    "main_net",
    enum_name="VerifierNetwork", 
)

SubmitProgramMeta = CStruct(
     "address" / U8[32],
     "network" / Verifier_Network,
     "git_repo" / String,
     "git_commit" / String,
     "directory" / String,
     "docker_version" / String,
     "rust_version" / String,
     "solana_version" / String,
     "anchor_version" / String
)

VerifyProgramMeta = CStruct(
    "test_address" / U8[32],
    "last_verified_slot" / U64,
    "verified" / U8,
    "data_hash" / U8[32],
    "code_meta" / U8[512]
)

StatusMeta = CStruct(
    "address" / U8[32],
    "status_code" / U8,
    "log_message" / String
)

Verifier_Instructions = Enum(
    "SubmitProgram" / SubmitProgramMeta,
    "VerifyProgram" / VerifyProgramMeta,
    "UpdateStatus" / StatusMeta,
    enum_name="VerifierInstruction", 
)

def network_to_u8(network):
    if(isinstance(network, Verifier_Network.enum.test_net)):
        return 0

    if(isinstance(network, Verifier_Network.enum.dev_net)):
        return 1

    if(isinstance(network, Verifier_Network.enum.main_net)):
        return 2

def network_to_string(network):
    if(isinstance(network, Verifier_Network.enum.test_net)):
        return "test_net"

    if(isinstance(network, Verifier_Network.enum.dev_net)):
        return "dev_net"

    if(isinstance(network, Verifier_Network.enum.main_net)):
        return "main_net"


class build_environment_t():

    def __init__(self, *args):

        self.rust_version = None
        self.solana_version = None
        self.anchor_version = None

        if (len(args) == 3):
            self.rust_version = args[0]
            self.solana_version = args[1]
            self.anchor_version = args[2]
  

    def build_from_args(self, args):
        if (args.rust_version != ""):
            self.rust_version = args.rust_version

        if (args.solana_version != ""):
            self.solana_version = args.solana_version

        if (args.anchor_version != ""):
            self.anchor_version = args.anchor_version

    def print(self):
        print("rust_version: ", self.rust_version)
        print("solana_version: ", self.solana_version)
        print("anchor_version: ", self.anchor_version)


# define a map of build environments for the prebuild dockers
BUILD_ENVIRONMENT_MAP = {}
BUILD_ENVIRONMENT_MAP["solana_v1.10.39"] = build_environment_t("1.63", "1.10.39", "0.25.0")