# verification codes start from 0
UNINITIALISED = 0  # we havn't run anything yet
MISMATCH = 1 # the provided data doesn't match the code on chain
MATCH_BUT_UPGRADEABLE = 2 # the code matches but the program can still be updated later
MATCH = 3 # the code matches and the program can't be upgraded
