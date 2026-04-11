 # Gitter http relay local test flow

  ## Terminal 1: start localnet + deploy gitter contracts
  cd apps/gitter && cargo run -p vastrum-cli -- run-dev 
                                                                                
  ## Terminal 2: start relay against that same localnet
  cd apps/gitter && VASTRUM_LOCALNET=1 cargo run -p vastrum-cli -- start-gitter-http-relay --relay-key relay.key                 
                                                                                
  ## Terminal 3: test git operations                                             
  git clone http://localhost:8080/example-repo                                  
  cd example-repo                                                               
  git commit --allow-empty -m "test"                                            
  git push ssh://git@localhost:2222/example-repo main
