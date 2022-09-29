# sol_verify
Solana program that handles verifying programs on chain with code in a git repo
The program is currently in early-access mode and a front end can be found at daoplays.org/verified

To do:

1) Handle the case that different users try and verify the same program:
    if a program hasn't passed a verification just let anyone try
    if a program has passed verification, but is upgradeable and has been upgraded since the last verification, set to unverified and allow anyone to try
    if a program has passed verification and isn't upgradeable, or hasn't been upgraded then only update if the verification still passes
    
2) Add support for security.txt to the program creators to specify the source location there, rather than having to enter it in the GUI.  Make this override anything entered into the GUI to stop people copying a git repo and then claiming credit for a particular program.

3) Store meta data for the program on chain in the programs's meta data account : code location, hash of the code etc

4) Add support for non git based code repositories

5) Integrate into explorers
