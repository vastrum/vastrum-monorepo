# Stateful Wallet



Some wallets are stateful and require the user to locally store state in order to spend their balance,
 for example Lightning Network or Tornado Cash. Some of the time this is solved by centralized storage by wallet providers.

For example Tornado Cash generates a 64 byte note you have to locally keep track of in a text file.

You could make a wallet that uses Vastrum to store encrypted notes, making stateful wallets stateless significantly increasing usability of note based payment systems.



