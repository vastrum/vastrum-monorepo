# Social Recovery

Currently the web-client generates a private key for the user and stores it in the web browsers local storage.


This has many problems
-   If for any reason the local storage is wiped, the private key is lost
-   To use same identity on another device, user needs to manually handle and copy private key


There are some ways to solve this
-   Allow user to use regular browser wallet extensions
    -   Only works with onboarded crypto people
-   Nag user to backup their private key

The best solution would be to replicate the web2 account authentication process
-   Email login
-   OAuth?
-   Passkeys?

However it is very difficult to achieve email account recovery in decentralized manner
-   At some point email needs to be in plaintext in order to send recovery email
-   The private key needs to be kept in custody

The centralized solution is very easy and is basically just the web2 solution, however it means a single entity controls the private keys for all users.

The best way would be to do this in decentralized manner where there are at least some security against any one actor stealing the private keys.

Using ZK email verification the private key custody problem can be solved.

The email custody risk still remains though and basically requires a centralized actor to hold custody.
-   To avoid censorship, could have some mechanism to decrypt all email addresses if the actor does not send recovery emails
-   Have the user hash the email address? Very easy to bruteforce guess emails and find preimage though.
-   The best would be some kind of FHE solution


In general i think credibly decentralized email based account recovery could be achieved.

Without email recovery the usability of all applications becomes very low as it is very easy to lose access to your account currently.