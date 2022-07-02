# Use cases and examples

Please read cases carefully - our fiat bridge supports anyone with a bank account to process
"direct payments" from other bank bank accounts,  wallet addresses and smart contracts.  
These use cases are currently not offered by the existing ecosystem - this requires 
new thinking about the topic of fiat on- and off-ramping.

## The Actors

- Alice: Operates an NFT-shop with a Fiat-Bridge where anyone can sell or buy NFTs with Fiat-Money. She "owns" the bridge, the bank account and set up access to the bank account.
- Bob: Wants to sell an NFT. He creates an account on Alice Platform, uploads his NFTs and ask for 5 Euros each.
- Charly: Wants to buy the NFT, he also created an account on Alice NFT platform.
- Dave: He likes the work of Bob - he has no web3 wallet and is not into crypto.
 He just wants to support Bob's work with real cash, and maybe claim the NFTs later. 

Remark: The bridge may or may not accept payments from Dave to to compliance reasons, this shows 
technical feasability that users do not have be registered for on- or off-ramping.

## Alice is setting up the Bridge

We assume alice already has 100 Euro on the bank account before setting up the Fita bridge for Alice NFT platform. After Alice sets up the bridge and after first time reading the bank-statement, here balance of pEURO is 100 and the total amount of pEURO is 100.

## Bob offers an NFT

Bob offers an NFT for 5 pEURO. He creates an account and maps his IBAN bank account to his wallet. The OCW
has his IBAN now stored. The balance of Bob's wallet is 0 pEURO.

## Charly does FIAT-onboarding - changes 50 EURO to 50 pEURO

Charly onboards with Alice NFT platform, add her bank account (IBAN) number, which maps with her wallet address to her IBAN on the OCW by calling the extrinsic 'fiatRamps.mapIbanAccount(iban)'. Now she wants to load her wallet with 50 pEURO. She sends 50 Euros to Alice via wire transfer. As the money arrives on Alice's bank account, following happens:

- The OCW sees the incoming transaction from Charly of 50 EURO and that the balance of the bank account is now 100 EURO.
- The total amount of pEURO should be soon 150, if everything is set up correctly.
- The OWC retrieves the wallet-address from Charly from his previous onboarding.
- The OWC mints 50 pEURO in Charly's wallet.

## Charly buys Bob's NFT

As Charly has now 50 pEURO he is able to buy the NFT via a blockchain transaction - following happens:

- Charly is on Alice's NFT platform and connects his wallet with 50 pEURO. He scrolls through the list of 
NFTs and sees Bob's fancy NFT.
- Charly clicks "buy" on Bobs NFT.
- This triggers the confirmation of a transaction of 5 pEURO from Charly to Bob.
- Bobs Balance goes from 0 pEURO to 5 pEURO.
- Charlys Balance goes from 50 pEURO to 45 pEURO.

The Bridge or bank accounts are not involved. The total balance is still unchanged at 150 pEURO, 
and also bank account balance is at 150 EURO.

Important Remark: Charly could have send Fiat funds directly to Bob as well - without loading her wallet.
Check out Dave's case.

## Bob does a "cash-out" - off-ramp

Charly now has 5 pEUROs. He is able to burn his pEUROs, thus triggering an off-ramp event.
He connects his wallet to Alice NFT platform and hits the "burn" button, select 5 pEURO as amount. 
Following happens:

- Charly has to sign the burn transaction.
- The OCW locks the pEURO intermediary.
- The OCW looks up Bobs bank account number (IBAN) triggers a bank transaction on Alice bank account
to Bobs IBAN.

The OCW keeps polling the bank account of Alice. As the OCW sees the transaction of 5 EUROs are 
listed in the daily statement following happens:

- Bank account balance is now at 145 EURO.
- OWC burns the 5 locked pEURO, total amount of pEURO is at 145 as well. 
- The OWC marks the off-ramping of 5 pEURO as completed. 

## Dave is sending Cash to Bob without having a wallet

Dave scrolls through Alice's platform and really likes Bob's work. He wants to donate 
10 EUROs. Be he can not connect
a Wallet to transer pEUROs because he does not know how to do it. So instead of signing a
transaction, he uses the QR-code of Alice Platform and sends a wire transfer from his
bank account. Following happens:

- Dave is scanning the QR code for the wire transfer (or enters payment details manually).
- He confirms the wire transfer in his banking app.
- Dave sends 10 EUROs from his bank account to Alice bank account.

The OCW keeps polling the bank account of Alice. As the OCW sees the transaction of 10 EUROs are 
listed in the daily statement following happens:

- The bank account balance is now at 155 EURO (145 + 10).
- The OCW sees the new incoming transaction of 10 EUROs from Dave - but the OWC does not know 
Dave, because he never registered on the NFT platform.
- The OCW does not know Daves IBAN, so it does not to which address it should send
  the 10 pEUROs. But the QR code for the bank transaction contains a code in reference field 
  of the bank transaction which points to Bobs wallet address. 
- The OCW mints 10 pEURO into Bobs wallet. 
- Total amount of pEURO is now 155.

Remark: The OWC has recorded the transfer of funds. If Dave later on is able to work
with web3 wallets and to connect his wallet to the NFT platform, he would be able
to claim a discount or NFT tokens based on his Fiat payment, if he connects his
IBAN account with his web3 account by onboarding to Alice platform.  But a 
lot more use cases are possbile combining wire transfers with on-chain 
smart contracts or web3 wallets. Basically the fiat bridge akts as a gateway to the fiat
world but also as an oracle. You can proove that fiat funds where sent and build
smart contracts on these facts. 