# Search Engine and Discoverability



A native search engine could be implemented on Vastrum.

There are multiple approaches to this, the one i am most interested in is local semantic search.

Basically embed a semantic search model in the frontend, calculate the query vector on the query locally.

Then have all "scraped" Vastrum websites embedded using same semantic search model.

To find the closest scraped website, the client then does some kind of a binary search to find the closest scraped website.


Very handwavy, but interesting if it could be done. 

Alternatives is to have some kind of curation model and not support free text search. 
ie top 50 sites in defi frontends, top 50 sites in forum categories.