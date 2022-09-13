our client needs to communicate with other client's using [[peer to peer network]] and transfer finished [[blockchain]]s

we want to constantly listen for [[blockchain]]s
once we receive a longer [[blockchain]] than the one we already have stored we verify all the [[transaction]]s, timestamps, hashes, if valid then we replace ours and start mining new block with [[transaction]]s not yet verified by the new [[blockchain]]

we want to mine a block that is constantly changing because of incoming [[transaction]]s, or changing [[blockchain]], we use mutex
however either way once our [[block]] is mined we want to append it to [[blockchain]] and publish it on the network

steps:
	1. establish communication with peers
	2. start mining empty mutable block with coinbase [[transaction]]
	3. listen for incoming [[blockchain]]
	4. listen for [[transaction]]s