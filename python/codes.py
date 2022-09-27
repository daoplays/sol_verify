# verification codes start from 0
UNINITIALISED = 0  # we havn't run anything yet
MISMATCH = 1 # the provided data doesn't match the code on chain
MATCH_BUT_UPGRADEABLE = 2 # the code matches but the program can still be updated later
MATCH = 3 # the code matches and the program can't be upgraded

# error codes
PROGRAM_DOESNT_EXIST = 100
GIT_REPO_DOESNT_EXIST = 101
GIT_DIR_DOESNT_EXIST = 102
GIT_COMMIT_DOESNT_EXIST = 103
UNSUPPORTED_LANGUAGE = 104
BUILD_FAILED = 105,
DOCKER_DOESNT_EXIST = 106
DOCKER_BUILD_FAILED = 107