# sol_verify
Solana program that handles verifying programs on chain with code in a git repo
The program is currently in early-access mode and a front end can be found at daoplays.org/verified

To do:

0) Add support for Anchor rust projects

1) Add support for specifying components of a docker image (ie rust version, solana version etc) rather than just selecting a docker image.  The backend would then build the required docker image itself.

2) Handle the case that different users try and verify the same program:
    if a program hasn't passed a verification just let anyone try
    if a program has passed verification, but is upgradeable and has been upgraded since the last verification, set to unverified and allow anyone to try
    if a program has passed verification and isn't upgradeable, or hasn't been upgraded then only update if the verification still passes
    
3) Add support for security.txt to the program creators to specify the source location there, rather than having to enter it in the GUI.  Make this override anything entered into the GUI to stop people copying a git repo and then claiming credit for a particular program.

4) Store meta data for the program on chain in the programs's meta data account : code location, hash of the code etc

5) Make it easier for a user to just check the current verification status of a program and any meta data that exists.

6) Add support for non git based code repositories

7) Add support for non rust based programs

8) Support main-net apps

9) Integrate into explorers
