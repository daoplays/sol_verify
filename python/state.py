from borsh_construct import Enum, CStruct, String, U64, U8
import base58

PROGRAM_KEY = "4xTTRRsDAjme4JoZxQ87czQvmstZ6onJJdNAQXpPw9PA"

def print_submit_meta(meta):
    print("address: ", base58.b58encode(bytearray(meta.address)).decode("utf-8"))
    print("git_repo: ", meta.git_repo)
    print("git_commit: ", meta.git_commit)
    print("directory: ", meta.directory)
    print("docker_version: ", meta.docker_version)

SubmitProgramMeta = CStruct(
     "address" / U8[32],
     "git_repo" / String,
     "git_commit" / String,
     "directory" / String,
     "docker_version" / String
)

VerifyProgramMeta = CStruct(
    "verified" / U8,
    "real_address" / U8[32],
    "test_address" / U8[32],
    "data_hash" / U8[32],
    "verified_slot" / U64
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

