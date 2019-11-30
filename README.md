# Origin Protocol built as a Substrate Runtime

Every 2-sided marketplace has some common denominators. Origin Substrate extracts those common denominators out to the base runtime, while giving marketplace creators the ability to write domain specific rules a la smart contracts.

#### Example (Airbnb vs. Uber vs. Tinder)
What do Airbnb, Uber, Ebay, and Tinder have in common? Well, yes they are 2 sided marketplaces. Which means:

1. Two Mutually Benefitting Entity Groups: i.e. Airbnb (Host & Guest), Uber (Rider & Driver), Ebay (Buyer & Seller), Tinder (Woman & Man)

2. Network Effects: Buyers want a network with many sellers (competition to drive down prices), while Sellers want a network with many buyers (demand to drive up prices). The result is that users on either side prefer to conduct their business in a bustling marketplace than in a sparse one. Users will pay a highe price to access a larger network.

3. Match Making: The Multi-Sided Platform is in the business of efficiently matching members of the supply side to those on the demand side, and extracting a service fee. The core service of any multi-sided platform is in the match-making.

### Why as a runtime?
SRMLs give flexibilty over Ethereum smart contracts in:
* Forkless upgradability
* Onchain governance out of the box
* Ability to add staking as incentive mechanism
* Ability to add specialized rules for any particular marketplace launched under the core Origin Protocol
* Access to off-chain data via offchain workers
* Proveable Finality with Grandpa
* Easy onramp into Polkadot and its shared security model with Cumulus
* Wont be unintentionally rate-limited by high transaction volume by some other dapp on the same chain (as with Ethereum)

### Why Contracts?
While the core of the protocol is to render match making services, each marketplace has its own "house rules," so to speak.

